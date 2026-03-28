use std::{collections::HashMap, sync::OnceLock};

use windows::{
  core::{w, ComInterface},
  Graphics::{
    Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem},
    DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
  },
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
    Graphics::{
      Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0},
      Direct3D11::{
        D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext,
        ID3D11Texture2D, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC,
      },
      DirectComposition::{
        DCompositionCreateDevice2, IDCompositionDesktopDevice,
        IDCompositionSurface, IDCompositionTarget, IDCompositionVisual2,
        IDCompositionVisual3,
      },
      Dxgi::{
        Common::{
          DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM,
        },
        IDXGIDevice,
      },
    },
    System::WinRT::{
      Direct3D11::{
        CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
      },
      Graphics::Capture::IGraphicsCaptureItemInterop,
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, GetSystemMetrics,
      RegisterClassW, ShowWindow, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
      SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_HIDE, SW_SHOWNA, WNDCLASSW,
      WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOPMOST,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{
  platform_impl::com::COM_INIT, Dispatcher, LayerId, NativeWindow,
  OpacityValue, Rect,
};

/// Guard ensuring the overlay window class is registered at most once.
static OVERLAY_CLASS: OnceLock<()> = OnceLock::new();

/// Maximum time to wait for a WGC frame before giving up.
const WGC_CAPTURE_TIMEOUT: std::time::Duration =
  std::time::Duration::from_secs(1);

/// Polling interval when waiting for a WGC frame.
const WGC_POLL_INTERVAL: std::time::Duration =
  std::time::Duration::from_millis(10);

/// State for a single animation layer within the surface.
struct LayerState {
  visual: IDCompositionVisual3,
  src_width: u32,
  src_height: u32,
}

/// Platform-specific implementation of [`AnimationSurface`].
///
/// Uses a single overlay HWND with DirectComposition for GPU-composited
/// rendering and Windows.Graphics.Capture for window screenshots.
pub(crate) struct AnimationSurface {
  hwnd: isize,
  _d3d_device: ID3D11Device,
  d3d_context: ID3D11DeviceContext,
  dcomp_device: IDCompositionDesktopDevice,
  _dcomp_target: IDCompositionTarget,
  root_visual: IDCompositionVisual2,
  /// WinRT device wrapper needed by the WGC frame pool.
  winrt_device: IDirect3DDevice,
  /// Dispatcher for running HWND operations on the event loop thread.
  dispatcher: Dispatcher,
  /// Virtual screen origin (top-left of all monitors combined).
  vx: i32,
  vy: i32,
  layers: HashMap<LayerId, LayerState>,
  next_id: u64,
}

impl AnimationSurface {
  /// Creates the D3D11/DirectComposition device stack and the single
  /// overlay HWND spanning all monitors.
  ///
  /// The overlay HWND is created on the event loop thread so that its
  /// messages are pumped by the main message loop.
  pub(crate) fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    COM_INIT.with(|com_init| {
      let (d3d_device, d3d_context) = create_d3d11_device()?;

      let dxgi_device: IDXGIDevice = d3d_device.cast()?;
      let dcomp_device: IDCompositionDesktopDevice =
        unsafe { DCompositionCreateDevice2(&dxgi_device)? };

      let winrt_device = create_winrt_device(&dxgi_device)?;

      // Get bounding rectangle of all monitors.
      let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
      let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
      let cx = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
      let cy = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };

      let hwnd = dispatcher
        .dispatch_sync(move || create_overlay_hwnd(vx, vy, cx, cy))??;

      let dcomp_target =
        unsafe { dcomp_device.CreateTargetForHwnd(HWND(hwnd), true)? };

      let root_visual = unsafe { dcomp_device.CreateVisual()? };
      unsafe { dcomp_target.SetRoot(&root_visual)? };
      unsafe { dcomp_device.Commit()? };

      Ok(Self {
        hwnd,
        _d3d_device: d3d_device,
        d3d_context,
        dcomp_device,
        _dcomp_target: dcomp_target,
        root_visual,
        winrt_device,
        dispatcher: dispatcher.clone(),
        vx,
        vy,
        layers: HashMap::new(),
        next_id: 0,
      })
    })
  }

  /// Captures a screenshot of `window` via WGC, creates a
  /// DirectComposition visual, and adds it to the root visual tree.
  pub(crate) fn add_layer(
    &mut self,
    window: &NativeWindow,
    rect: &Rect,
    opacity: Option<f32>,
  ) -> crate::Result<LayerId> {
    COM_INIT.with(|com_init| {
      let captured_texture =
        capture_window_texture(window.inner.hwnd(), &self.winrt_device)?;

      let mut desc = D3D11_TEXTURE2D_DESC::default();
      unsafe { captured_texture.GetDesc(&raw mut desc) };

      let dcomp_surface = unsafe {
        self.dcomp_device.CreateSurface(
          desc.Width,
          desc.Height,
          DXGI_FORMAT_B8G8R8A8_UNORM,
          DXGI_ALPHA_MODE_PREMULTIPLIED,
        )?
      };

      copy_texture_to_surface(
        &self.d3d_context,
        &captured_texture,
        &dcomp_surface,
      )?;

      let visual: IDCompositionVisual3 =
        unsafe { self.dcomp_device.CreateVisual()?.cast()? };

      unsafe { visual.SetContent(&dcomp_surface)? };

      #[allow(clippy::cast_precision_loss)]
      unsafe {
        visual.SetOffsetX2((rect.x() - self.vx) as f32)?;
        visual.SetOffsetY2((rect.y() - self.vy) as f32)?;
      }

      apply_scale(
        &self.dcomp_device,
        &visual,
        rect.width(),
        rect.height(),
        desc.Width,
        desc.Height,
      )?;

      if let Some(alpha) = opacity {
        unsafe { visual.SetOpacity2(alpha)? };
      }

      unsafe {
        self.root_visual.AddVisual(&visual, true, None)?;
        self.dcomp_device.Commit()?;
      }

      let id = LayerId(self.next_id);
      self.next_id += 1;

      self.layers.insert(
        id,
        LayerState {
          visual,
          src_width: desc.Width,
          src_height: desc.Height,
        },
      );

      Ok(id)
    })
  }

  /// Updates frame position, size, and opacity for active layers, then
  /// commits all changes in a single DirectComposition transaction.
  pub(crate) fn update_layers(
    &self,
    updates: Vec<(LayerId, Rect, Option<OpacityValue>)>,
  ) -> crate::Result<()> {
    COM_INIT.with(|com_init| {
      for (id, rect, opacity) in &updates {
        if let Some(layer) = self.layers.get(id) {
          #[allow(clippy::cast_precision_loss)]
          unsafe {
            layer.visual.SetOffsetX2((rect.x() - self.vx) as f32)?;
            layer.visual.SetOffsetY2((rect.y() - self.vy) as f32)?;
          }

          apply_scale(
            &self.dcomp_device,
            &layer.visual,
            rect.width(),
            rect.height(),
            layer.src_width,
            layer.src_height,
          )?;

          if let Some(opacity) = opacity {
            unsafe { layer.visual.SetOpacity2(opacity.0)? };
          }
        }
      }

      unsafe { self.dcomp_device.Commit()? };
      Ok(())
    })
  }

  /// Removes a layer's visual from the composition tree.
  pub(crate) fn remove_layer(&mut self, id: LayerId) -> crate::Result<()> {
    COM_INIT.with(|com_init| {
      if let Some(layer) = self.layers.remove(&id) {
        if let Err(err) =
          unsafe { self.root_visual.RemoveVisual(&layer.visual) }
        {
          tracing::warn!("Failed to remove visual from root: {}", err);
        }
        unsafe { self.dcomp_device.Commit()? };
      }

      Ok(())
    })
  }

  /// Returns whether the surface has any active layers.
  pub(crate) fn has_layers(&self) -> crate::Result<bool> {
    Ok(!self.layers.is_empty())
  }

  /// Shows the overlay window (no-activate).
  pub(crate) fn show(&self) -> crate::Result<()> {
    let hwnd = self.hwnd;
    self.dispatcher.dispatch_sync(move || {
      let _ = unsafe { ShowWindow(HWND(hwnd), SW_SHOWNA) };
    })?;
    Ok(())
  }

  /// Hides the overlay window without destroying it.
  pub(crate) fn hide(&self) -> crate::Result<()> {
    let hwnd = self.hwnd;
    self.dispatcher.dispatch_sync(move || {
      let _ = unsafe { ShowWindow(HWND(hwnd), SW_HIDE) };
    })?;
    Ok(())
  }

  /// Destroys the surface, removing all visuals and the overlay HWND.
  pub(crate) fn destroy(mut self) -> crate::Result<()> {
    for (_, layer) in self.layers.drain() {
      let _ = unsafe { self.root_visual.RemoveVisual(&layer.visual) };
    }

    let _ = unsafe { self.dcomp_device.Commit() };

    let hwnd = self.hwnd;
    self.dispatcher.dispatch_sync(move || {
      let _ = unsafe { DestroyWindow(HWND(hwnd)) };
    })?;

    Ok(())
  }

  /// Window procedure for the overlay class.
  unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
  }
}

/// Creates a D3D11 device with BGRA support (required by
/// DirectComposition).
fn create_d3d11_device(
) -> crate::Result<(ID3D11Device, ID3D11DeviceContext)> {
  let mut device: Option<ID3D11Device> = None;
  let mut context: Option<ID3D11DeviceContext> = None;

  unsafe {
    D3D11CreateDevice(
      None,
      D3D_DRIVER_TYPE_HARDWARE,
      None,
      D3D11_CREATE_DEVICE_BGRA_SUPPORT,
      Some(&[D3D_FEATURE_LEVEL_11_0]),
      D3D11_SDK_VERSION,
      Some(&raw mut device),
      None,
      Some(&raw mut context),
    )?;
  }

  let device = device.ok_or_else(|| {
    crate::Error::Platform(
      "D3D11CreateDevice returned null device.".to_string(),
    )
  })?;

  let context = context.ok_or_else(|| {
    crate::Error::Platform(
      "D3D11CreateDevice returned null context.".to_string(),
    )
  })?;

  Ok((device, context))
}

/// Converts a DXGI device into a WinRT `IDirect3DDevice` for use with
/// the WGC frame pool.
fn create_winrt_device(
  dxgi_device: &IDXGIDevice,
) -> crate::Result<IDirect3DDevice> {
  let inspectable =
    unsafe { CreateDirect3D11DeviceFromDXGIDevice(dxgi_device)? };
  Ok(inspectable.cast()?)
}

/// Creates the single layered overlay HWND spanning all monitors.
fn create_overlay_hwnd(
  x: i32,
  y: i32,
  width: i32,
  height: i32,
) -> crate::Result<isize> {
  OVERLAY_CLASS.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("AnimationOverlay"),
      lpfnWndProc: Some(AnimationSurface::overlay_wnd_proc),
      ..Default::default()
    };
    unsafe { RegisterClassW(&raw const wnd_class) };
  });

  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_NOREDIRECTIONBITMAP
        | WS_EX_TOPMOST
        | WS_EX_NOACTIVATE
        | WS_EX_TRANSPARENT,
      w!("AnimationOverlay"),
      w!(""),
      WS_POPUP,
      x,
      y,
      width,
      height,
      None,
      None,
      None,
      None,
    )
  };

  if hwnd.0 == 0 {
    return Err(crate::Error::Platform(
      "Failed to create overlay window.".to_string(),
    ));
  }

  Ok(hwnd.0)
}

/// Captures a single frame of a window via Windows.Graphics.Capture and
/// returns its `ID3D11Texture2D`.
fn capture_window_texture(
  hwnd: HWND,
  winrt_device: &IDirect3DDevice,
) -> crate::Result<ID3D11Texture2D> {
  if !windows::Graphics::Capture::GraphicsCaptureSession::IsSupported()? {
    return Err(crate::Error::Platform(
      "Windows.Graphics.Capture is not supported on this system."
        .to_string(),
    ));
  }

  let interop: IGraphicsCaptureItemInterop = windows::core::factory::<
    GraphicsCaptureItem,
    IGraphicsCaptureItemInterop,
  >()?;

  // SAFETY: HWND is valid per the caller's contract.
  let capture_item: GraphicsCaptureItem =
    unsafe { interop.CreateForWindow(hwnd)? };

  let item_size = capture_item.Size()?;

  let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
    winrt_device,
    DirectXPixelFormat::B8G8R8A8UIntNormalized,
    1,
    item_size,
  )?;

  let session = frame_pool.CreateCaptureSession(&capture_item)?;

  // Disable the yellow capture border on Windows 11+.
  let _ = session.SetIsBorderRequired(false);

  session.StartCapture()?;

  let frame = poll_for_frame(&frame_pool)?;

  let surface = frame.Surface()?;
  let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;

  // SAFETY: The WGC surface wraps a valid D3D11 texture.
  let texture: ID3D11Texture2D = unsafe { access.GetInterface()? };

  // WGC textures are pool-managed — copy into a standalone texture
  // to avoid use-after-free when the pool is closed.
  let standalone = copy_to_standalone_texture(&texture)?;

  session.Close()?;
  frame_pool.Close()?;

  Ok(standalone)
}

/// Polls `TryGetNextFrame` until a frame arrives or the timeout expires.
fn poll_for_frame(
  pool: &Direct3D11CaptureFramePool,
) -> crate::Result<windows::Graphics::Capture::Direct3D11CaptureFrame> {
  let deadline = std::time::Instant::now() + WGC_CAPTURE_TIMEOUT;

  loop {
    match pool.TryGetNextFrame() {
      Ok(frame) => return Ok(frame),
      Err(_) if std::time::Instant::now() < deadline => {
        std::thread::sleep(WGC_POLL_INTERVAL);
      }
      Err(err) => {
        return Err(crate::Error::Platform(format!(
          "WGC capture timed out: {err}"
        )));
      }
    }
  }
}

/// Creates a standalone copy of a pool-managed WGC texture.
fn copy_to_standalone_texture(
  src: &ID3D11Texture2D,
) -> crate::Result<ID3D11Texture2D> {
  let mut desc = D3D11_TEXTURE2D_DESC::default();
  unsafe { src.GetDesc(&raw mut desc) };

  desc.BindFlags = 0;
  desc.MiscFlags = 0;
  desc.CPUAccessFlags = 0;

  let device = unsafe { src.GetDevice()? };
  let mut copy: Option<ID3D11Texture2D> = None;
  unsafe {
    device.CreateTexture2D(&raw const desc, None, Some(&raw mut copy))?;
  }

  let copy = copy.ok_or_else(|| {
    crate::Error::Platform("CreateTexture2D returned null.".to_string())
  })?;

  let context = unsafe { device.GetImmediateContext()? };
  unsafe { context.CopyResource(&copy, src) };

  Ok(copy)
}

/// Copies the contents of `src_texture` into a DComp `surface` via
/// `BeginDraw`/`EndDraw`.
///
/// `BeginDraw` may return an atlas texture larger than the surface, so
/// `CopySubresourceRegion` is used with the returned offset rather than
/// `CopyResource` (which requires matching dimensions).
fn copy_texture_to_surface(
  context: &ID3D11DeviceContext,
  src_texture: &ID3D11Texture2D,
  surface: &IDCompositionSurface,
) -> crate::Result<()> {
  let mut offset = POINT::default();

  let update_texture: ID3D11Texture2D =
    unsafe { surface.BeginDraw(None, &raw mut offset)? };

  #[allow(clippy::cast_sign_loss)]
  unsafe {
    context.CopySubresourceRegion(
      &update_texture,
      0,
      offset.x as u32,
      offset.y as u32,
      0,
      src_texture,
      0,
      None,
    );
  }

  unsafe { surface.EndDraw()? };

  Ok(())
}

/// Applies a scale transform on `visual` so the captured source is
/// scaled to fill the target rect dimensions.
fn apply_scale(
  dcomp_device: &IDCompositionDesktopDevice,
  visual: &IDCompositionVisual3,
  target_width: i32,
  target_height: i32,
  src_width: u32,
  src_height: u32,
) -> crate::Result<()> {
  if src_width == 0 || src_height == 0 {
    return Ok(());
  }

  #[allow(clippy::cast_precision_loss)]
  let scale_x = target_width as f32 / src_width as f32;
  #[allow(clippy::cast_precision_loss)]
  let scale_y = target_height as f32 / src_height as f32;

  let transform = unsafe { dcomp_device.CreateScaleTransform()? };
  unsafe {
    transform.SetScaleX2(scale_x)?;
    transform.SetScaleY2(scale_y)?;
    transform.SetCenterX2(0.0)?;
    transform.SetCenterY2(0.0)?;
    visual.SetTransform(&transform)?;
  }

  Ok(())
}
