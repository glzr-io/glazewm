use std::sync::OnceLock;

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
        IDCompositionSurface, IDCompositionTarget, IDCompositionVisual3,
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
      CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW,
      SetWindowPos, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
      SWP_SHOWWINDOW, WNDCLASSW, WS_EX_NOACTIVATE,
      WS_EX_NOREDIRECTIONBITMAP, WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{
  platform_impl::com::COM_INIT, Dispatcher, NativeWindow, OpacityValue,
  Rect,
};

/// Guard ensuring the overlay window class is registered at most once.
static OVERLAY_CLASS: OnceLock<()> = OnceLock::new();

/// Maximum time to wait for a WGC frame before giving up.
const WGC_CAPTURE_TIMEOUT: std::time::Duration =
  std::time::Duration::from_secs(1);

/// Polling interval when waiting for a WGC frame.
const WGC_POLL_INTERVAL: std::time::Duration =
  std::time::Duration::from_millis(10);

/// Per-window overlay for animating a single window transition.
///
/// Each `AnimationWindow` creates its own transparent HWND sized to the
/// bounding box of the animation's start and target rects. The HWND is
/// ordered just above the source window via `SetWindowPos`, preserving
/// z-order among non-animated windows.
///
/// The contained DirectComposition visual is repositioned each tick; the
/// HWND frame stays fixed for the lifetime of the animation.
pub(crate) struct AnimationWindow {
  hwnd: isize,
  dcomp_device: IDCompositionDesktopDevice,
  _dcomp_target: IDCompositionTarget,
  visual: IDCompositionVisual3,
  /// Captured source texture dimensions (for scale calculations).
  src_width: u32,
  src_height: u32,
  /// Top-left of the overlay HWND in screen coordinates.
  origin_x: i32,
  origin_y: i32,
  /// Dispatcher for running HWND operations on the event loop thread.
  dispatcher: Dispatcher,
}

impl AnimationWindow {
  /// Creates a transparent overlay HWND covering the union of
  /// `start_rect` and `target_rect`, captures a screenshot of `window`
  /// via WGC, and orders the overlay just above the source window.
  pub(crate) fn new(
    dispatcher: &Dispatcher,
    window: &NativeWindow,
    start_rect: &Rect,
    target_rect: &Rect,
    opacity: Option<f32>,
  ) -> crate::Result<Self> {
    COM_INIT.with(|_| {
      let (d3d_device, d3d_context) = create_d3d11_device()?;

      let dxgi_device: IDXGIDevice = d3d_device.cast()?;
      let dcomp_device: IDCompositionDesktopDevice =
        unsafe { DCompositionCreateDevice2(&dxgi_device)? };

      let winrt_device = create_winrt_device(&dxgi_device)?;

      let captured_texture =
        capture_window_texture(window.inner.hwnd(), &winrt_device)?;

      let mut desc = D3D11_TEXTURE2D_DESC::default();
      unsafe { captured_texture.GetDesc(&raw mut desc) };

      let bounds = start_rect.union(target_rect);
      let origin_x = bounds.x();
      let origin_y = bounds.y();
      let source_hwnd = window.inner.hwnd().0;

      let hwnd = dispatcher.dispatch_sync(move || {
        create_overlay_hwnd(
          source_hwnd,
          origin_x,
          origin_y,
          bounds.width(),
          bounds.height(),
        )
      })??;

      let dcomp_target =
        unsafe { dcomp_device.CreateTargetForHwnd(HWND(hwnd), true)? };

      let dcomp_surface = unsafe {
        dcomp_device.CreateSurface(
          desc.Width,
          desc.Height,
          DXGI_FORMAT_B8G8R8A8_UNORM,
          DXGI_ALPHA_MODE_PREMULTIPLIED,
        )?
      };

      copy_texture_to_surface(
        &d3d_context,
        &captured_texture,
        &dcomp_surface,
      )?;

      let visual: IDCompositionVisual3 =
        unsafe { dcomp_device.CreateVisual()?.cast()? };

      unsafe {
        visual.SetContent(&dcomp_surface)?;
        dcomp_target.SetRoot(&visual)?;
      }

      #[allow(clippy::cast_precision_loss)]
      unsafe {
        visual.SetOffsetX2((start_rect.x() - origin_x) as f32)?;
        visual.SetOffsetY2((start_rect.y() - origin_y) as f32)?;
      }

      apply_scale(
        &dcomp_device,
        &visual,
        start_rect.width(),
        start_rect.height(),
        desc.Width,
        desc.Height,
      )?;

      if let Some(alpha) = opacity {
        unsafe { visual.SetOpacity2(alpha)? };
      }

      unsafe { dcomp_device.Commit()? };

      Ok(Self {
        hwnd,
        dcomp_device,
        _dcomp_target: dcomp_target,
        visual,
        src_width: desc.Width,
        src_height: desc.Height,
        origin_x,
        origin_y,
        dispatcher: dispatcher.clone(),
      })
    })
  }

  /// Resizes the overlay HWND to cover the union of `start_rect` and
  /// `target_rect`, updating the stored origin.
  ///
  /// Called when an animation's target changes mid-flight so the
  /// existing screenshot and z-order are preserved.
  pub(crate) fn resize(
    &mut self,
    start_rect: &Rect,
    target_rect: &Rect,
  ) -> crate::Result<()> {
    let bounds = start_rect.union(target_rect);

    self.origin_x = bounds.x();
    self.origin_y = bounds.y();

    let hwnd = self.hwnd;
    let x = bounds.x();
    let y = bounds.y();
    let w = bounds.width();
    let h = bounds.height();

    unsafe {
      SetWindowPos(HWND(hwnd), None, x, y, w, h, SWP_NOACTIVATE)?;
    }

    Ok(())
  }

  /// Repositions the DirectComposition visual within the overlay HWND
  /// and updates opacity.
  ///
  /// The HWND frame is never changed; only the visual moves.
  pub(crate) fn update(
    &self,
    rect: &Rect,
    opacity: Option<OpacityValue>,
  ) -> crate::Result<()> {
    COM_INIT.with(|_| {
      #[allow(clippy::cast_precision_loss)]
      unsafe {
        self.visual.SetOffsetX2((rect.x() - self.origin_x) as f32)?;
        self.visual.SetOffsetY2((rect.y() - self.origin_y) as f32)?;
      }

      apply_scale(
        &self.dcomp_device,
        &self.visual,
        rect.width(),
        rect.height(),
        self.src_width,
        self.src_height,
      )?;

      if let Some(opacity) = opacity {
        unsafe { self.visual.SetOpacity2(opacity.0)? };
      }

      unsafe { self.dcomp_device.Commit()? };
      Ok(())
    })
  }

  /// Destroys the overlay HWND.
  pub(crate) fn destroy(self) -> crate::Result<()> {
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

/// Creates the overlay HWND for a single animation, sized to the given
/// rect and ordered just above `source_hwnd`.
fn create_overlay_hwnd(
  source_hwnd: isize,
  x: i32,
  y: i32,
  width: i32,
  height: i32,
) -> crate::Result<isize> {
  OVERLAY_CLASS.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("AnimationOverlay"),
      lpfnWndProc: Some(AnimationWindow::overlay_wnd_proc),
      ..Default::default()
    };
    unsafe { RegisterClassW(&raw const wnd_class) };
  });

  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_NOREDIRECTIONBITMAP | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT,
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

  // Order the overlay just above the source window and show it.
  unsafe {
    SetWindowPos(
      hwnd,
      HWND(source_hwnd),
      0,
      0,
      0,
      0,
      SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
    )?;
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

  // WGC textures are pool-managed -- copy into a standalone texture
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
