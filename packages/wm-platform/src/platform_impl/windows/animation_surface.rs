use std::{
  collections::HashMap,
  sync::{mpsc, OnceLock},
};

use windows::{
  core::{w, ComInterface, IInspectable},
  Foundation::TypedEventHandler,
  Graphics::{
    Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem},
    DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
  },
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM},
    Graphics::{
      Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0},
      Direct3D11::{
        D3D11CreateDevice, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, ID3D11Device,
        ID3D11DeviceContext, ID3D11Texture2D,
      },
      DirectComposition::{
        DCompositionCreateDevice, IDCompositionDevice,
        IDCompositionEffectGroup, IDCompositionScaleTransform,
        IDCompositionSurface, IDCompositionTarget, IDCompositionTransform,
        IDCompositionTranslateTransform, IDCompositionVisual,
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
        CreateDirect3D11DeviceFromDXGIDevice,
        IDirect3DDxgiInterfaceAccess,
      },
      Graphics::Capture::IGraphicsCaptureItemInterop,
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, GetSystemMetrics,
      RegisterClassW, ShowWindow, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
      SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SW_HIDE, SW_SHOWNA,
      WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOPMOST,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{
  animation_surface::LayerId, Dispatcher, NativeWindow, OpacityValue,
  Rect,
};

/// Guard ensuring the overlay window class is registered at most once.
static OVERLAY_CLASS: OnceLock<()> = OnceLock::new();

/// Maximum time to wait for a WGC frame before giving up.
const WGC_CAPTURE_TIMEOUT: std::time::Duration =
  std::time::Duration::from_secs(1);

/// State for a single animation layer within the surface.
struct LayerState {
  visual: IDCompositionVisual,
  translate_transform: IDCompositionTranslateTransform,
  scale_transform: IDCompositionScaleTransform,
  effect_group: IDCompositionEffectGroup,
  src_width: u32,
  src_height: u32,
}

/// Platform-specific implementation of [`AnimationSurface`].
///
/// Uses a single overlay HWND with DirectComposition for GPU-composited
/// rendering and Windows.Graphics.Capture for window screenshots.
pub(crate) struct AnimationSurface {
  /// Overlay `HWND` stored as `isize` (`HWND` is `!Send`).
  hwnd: isize,
  _d3d_device: ID3D11Device,
  d3d_context: ID3D11DeviceContext,
  dcomp_device: IDCompositionDevice,
  _dcomp_target: IDCompositionTarget,
  root_visual: IDCompositionVisual,
  /// WinRT device wrapper needed by the WGC frame pool.
  winrt_device: IDirect3DDevice,
  /// Virtual screen origin (top-left of all monitors combined).
  vx: i32,
  vy: i32,
  layers: HashMap<LayerId, LayerState>,
  next_id: u64,
}

// SAFETY: The only non-Send field is `hwnd` (stored as `isize`). All COM
// pointers from the `windows` crate are `Send + Sync`. The `hwnd` is only
// used for `ShowWindow`/`DestroyWindow` calls.
unsafe impl Send for AnimationSurface {}
// SAFETY: Interior state is only mutated through `&mut self` methods.
unsafe impl Sync for AnimationSurface {}

impl AnimationSurface {
  /// Creates the D3D11/DirectComposition device stack and the single
  /// overlay HWND spanning all monitors.
  pub(crate) fn new(_dispatcher: &Dispatcher) -> crate::Result<Self> {
    let (d3d_device, d3d_context) = create_d3d11_device()?;

    let dxgi_device: IDXGIDevice = d3d_device.cast()?;
    let dcomp_device: IDCompositionDevice =
      unsafe { DCompositionCreateDevice(&dxgi_device)? };

    let winrt_device = create_winrt_device(&dxgi_device)?;

    let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let cx = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let cy = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };

    let hwnd = create_overlay_hwnd(vx, vy, cx, cy)?;

    let dcomp_target = unsafe {
      dcomp_device.CreateTargetForHwnd(HWND(hwnd), true)?
    };

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
      vx,
      vy,
      layers: HashMap::new(),
      next_id: 0,
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
    let captured_texture = capture_window_texture(
      window.inner.hwnd(),
      &self.winrt_device,
    )?;

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

    let visual = unsafe { self.dcomp_device.CreateVisual()? };
    let translate_transform =
      unsafe { self.dcomp_device.CreateTranslateTransform()? };
    let scale_transform =
      unsafe { self.dcomp_device.CreateScaleTransform()? };
    let effect_group = unsafe { self.dcomp_device.CreateEffectGroup()? };

    let transform_group: IDCompositionTransform = unsafe {
      self.dcomp_device.CreateTransformGroup(&[
        Some(translate_transform.cast()?),
        Some(scale_transform.cast()?),
      ])?
    };

    unsafe {
      visual.SetContent(&dcomp_surface)?;
      visual.SetTransform(&transform_group)?;
      visual.SetEffect(&effect_group)?;
    }

    update_layer_properties(
      &visual,
      &translate_transform,
      &scale_transform,
      &effect_group,
      rect,
      self.vx,
      self.vy,
      desc.Width,
      desc.Height,
      opacity,
    )?;

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
        translate_transform,
        scale_transform,
        effect_group,
        src_width: desc.Width,
        src_height: desc.Height,
      },
    );

    Ok(id)
  }

  /// Updates frame position, size, and opacity for active layers, then
  /// commits all changes in a single DirectComposition transaction.
  pub(crate) fn update_layers(
    &self,
    updates: Vec<(LayerId, Rect, Option<OpacityValue>)>,
  ) -> crate::Result<()> {
    for (id, rect, opacity) in &updates {
      if let Some(layer) = self.layers.get(id) {
        update_layer_properties(
          &layer.visual,
          &layer.translate_transform,
          &layer.scale_transform,
          &layer.effect_group,
          rect,
          self.vx,
          self.vy,
          layer.src_width,
          layer.src_height,
          opacity.map(|value| value.0),
        )?;
      }
    }

    unsafe { self.dcomp_device.Commit()? };
    Ok(())
  }

  /// Removes a layer's visual from the composition tree.
  pub(crate) fn remove_layer(&mut self, id: LayerId) -> crate::Result<()> {
    if let Some(layer) = self.layers.remove(&id) {
      if let Err(err) =
        unsafe { self.root_visual.RemoveVisual(&layer.visual) }
      {
        tracing::warn!("Failed to remove visual from root: {}", err);
      }
      unsafe { self.dcomp_device.Commit()? };
    }

    Ok(())
  }

  /// Returns whether the surface has any active layers.
  pub(crate) fn has_layers(&self) -> crate::Result<bool> {
    Ok(!self.layers.is_empty())
  }

  /// Shows the overlay window (no-activate).
  pub(crate) fn show(&self) -> crate::Result<()> {
    let _ = unsafe { ShowWindow(HWND(self.hwnd), SW_SHOWNA) };
    Ok(())
  }

  /// Hides the overlay window without destroying it.
  pub(crate) fn hide(&self) -> crate::Result<()> {
    let _ = unsafe { ShowWindow(HWND(self.hwnd), SW_HIDE) };
    Ok(())
  }

  /// Destroys the surface, removing all visuals and the overlay HWND.
  pub(crate) fn destroy(mut self) -> crate::Result<()> {
    for (_, layer) in self.layers.drain() {
      let _ = unsafe { self.root_visual.RemoveVisual(&layer.visual) };
    }

    let _ = unsafe { self.dcomp_device.Commit() };
    unsafe { DestroyWindow(HWND(self.hwnd))? };

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
  let inspectable = unsafe {
    CreateDirect3D11DeviceFromDXGIDevice(dxgi_device)?
  };
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
      lpszClassName: w!("GlazeWMOverlay"),
      lpfnWndProc: Some(AnimationSurface::overlay_wnd_proc),
      ..Default::default()
    };
    unsafe { RegisterClassW(&raw const wnd_class) };
  });

  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_LAYERED
        | WS_EX_TOPMOST
        | WS_EX_NOACTIVATE
        | WS_EX_TRANSPARENT,
      w!("GlazeWMOverlay"),
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
  let (tx, rx) = mpsc::sync_channel(1);
  let handler =
    TypedEventHandler::<Direct3D11CaptureFramePool, IInspectable>::new(
    move |_, _| {
      let _ = tx.try_send(());
      Ok(())
    },
  );
  let token = frame_pool.FrameArrived(&handler)?;

  // Disable the yellow capture border on Windows 11+.
  let _ = session.SetIsBorderRequired(false);

  session.StartCapture()?;

  let frame = wait_for_frame(&frame_pool, &rx)?;

  let surface = frame.Surface()?;
  let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;

  // SAFETY: The WGC surface wraps a valid D3D11 texture.
  let texture: ID3D11Texture2D = unsafe { access.GetInterface()? };

  // WGC textures are pool-managed — copy into a standalone texture
  // to avoid use-after-free when the pool is closed.
  let standalone = copy_to_standalone_texture(&texture)?;

  frame_pool.RemoveFrameArrived(token)?;
  session.Close()?;
  frame_pool.Close()?;

  Ok(standalone)
}

/// Waits for a WGC frame arrival event, then retrieves the frame.
fn wait_for_frame(
  pool: &Direct3D11CaptureFramePool,
  rx: &mpsc::Receiver<()>,
) -> crate::Result<windows::Graphics::Capture::Direct3D11CaptureFrame> {
  match pool.TryGetNextFrame() {
    Ok(frame) => return Ok(frame),
    Err(_) => {}
  }

  rx.recv_timeout(WGC_CAPTURE_TIMEOUT).map_err(|err| {
    crate::Error::Platform(format!(
      "WGC capture timed out waiting for FrameArrived: {err}"
    ))
  })?;

  pool.TryGetNextFrame().map_err(|err| {
    crate::Error::Platform(format!(
      "WGC frame retrieval failed after FrameArrived: {err}"
    ))
  })
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
    device.CreateTexture2D(
      &raw const desc,
      None,
      Some(&raw mut copy),
    )?;
  }

  let copy = copy.ok_or_else(|| {
    crate::Error::Platform(
      "CreateTexture2D returned null.".to_string(),
    )
  })?;

  let context = unsafe { device.GetImmediateContext()? };
  unsafe { context.CopyResource(&copy, src) };

  Ok(copy)
}

/// Copies the contents of `src_texture` into a DComp `surface` via
/// `BeginDraw`/`EndDraw`.
fn copy_texture_to_surface(
  context: &ID3D11DeviceContext,
  src_texture: &ID3D11Texture2D,
  surface: &IDCompositionSurface,
) -> crate::Result<()> {
  let mut offset = POINT::default();

  let update_texture: ID3D11Texture2D = unsafe {
    surface.BeginDraw(None, &raw mut offset)?
  };

  unsafe { context.CopyResource(&update_texture, src_texture) };
  unsafe { surface.EndDraw()? };

  Ok(())
}

/// Updates a layer's translation, scale, and opacity state.
fn update_layer_properties(
  _visual: &IDCompositionVisual,
  translate_transform: &IDCompositionTranslateTransform,
  scale_transform: &IDCompositionScaleTransform,
  effect_group: &IDCompositionEffectGroup,
  rect: &Rect,
  vx: i32,
  vy: i32,
  src_width: u32,
  src_height: u32,
  opacity: Option<f32>,
) -> crate::Result<()> {
  if src_width == 0 || src_height == 0 {
    return Ok(());
  }

  #[allow(clippy::cast_precision_loss)]
  let offset_x = (rect.x() - vx) as f32;
  #[allow(clippy::cast_precision_loss)]
  let offset_y = (rect.y() - vy) as f32;
  #[allow(clippy::cast_precision_loss)]
  let scale_x = rect.width() as f32 / src_width as f32;
  #[allow(clippy::cast_precision_loss)]
  let scale_y = rect.height() as f32 / src_height as f32;

  unsafe {
    translate_transform.SetOffsetX2(offset_x)?;
    translate_transform.SetOffsetY2(offset_y)?;
    scale_transform.SetScaleX2(scale_x)?;
    scale_transform.SetScaleY2(scale_y)?;
    scale_transform.SetCenterX2(0.0)?;
    scale_transform.SetCenterY2(0.0)?;
    effect_group.SetOpacity2(opacity.unwrap_or(1.0))?;
  }

  Ok(())
}
