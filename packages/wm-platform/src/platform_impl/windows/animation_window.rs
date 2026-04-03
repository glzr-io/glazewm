use std::sync::OnceLock;

use windows::{
  core::{w, ComInterface},
  Foundation::TypedEventHandler,
  Graphics::{
    Capture::{
      Direct3D11CaptureFrame, Direct3D11CaptureFramePool,
      GraphicsCaptureItem, GraphicsCaptureSession,
    },
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
        IDCompositionScaleTransform, IDCompositionSurface,
        IDCompositionTarget, IDCompositionVisual3,
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

/// Shared GPU context for all animation windows.
///
/// Holds a single D3D11 device, `DirectComposition` device, and
/// pre-warmed WGC capture objects, avoiding the overhead of creating
/// heavyweight GPU/WGC objects per animation.
pub(crate) struct AnimationContext {
  _d3d_device: ID3D11Device,
  d3d_context: ID3D11DeviceContext,
  dcomp_device: IDCompositionDesktopDevice,
  winrt_device: IDirect3DDevice,
  capture_interop: IGraphicsCaptureItemInterop,
}

impl AnimationContext {
  /// Creates a shared D3D11 + `DirectComposition` device pair and
  /// pre-warms WGC capture objects (WinRT device, interop factory).
  pub(crate) fn new(_dispatcher: &Dispatcher) -> crate::Result<Self> {
    COM_INIT.with(|_| {
      if !GraphicsCaptureSession::IsSupported()? {
        return Err(crate::Error::Platform(
          "Windows.Graphics.Capture isn't supported on this system."
            .to_string(),
        ));
      }

      let (d3d_device, d3d_context) = Self::create_d3d11_device()?;
      let dxgi_device: IDXGIDevice = d3d_device.cast()?;
      let dcomp_device: IDCompositionDesktopDevice =
        unsafe { DCompositionCreateDevice2(&dxgi_device)? };

      let inspectable =
        unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)? };
      let winrt_device: IDirect3DDevice = inspectable.cast()?;

      let capture_interop = windows::core::factory::<
        GraphicsCaptureItem,
        IGraphicsCaptureItemInterop,
      >()?;

      Ok(Self {
        _d3d_device: d3d_device,
        d3d_context,
        dcomp_device,
        winrt_device,
        capture_interop,
      })
    })
  }

  /// Executes `update_fn` and then commits all pending `DirectComposition`
  /// changes to the compositor in a single batch.
  pub(crate) fn transaction<F, R>(&self, update_fn: F) -> crate::Result<R>
  where
    F: FnOnce() -> R,
  {
    COM_INIT.with(|_| {
      let result = update_fn();
      unsafe { self.dcomp_device.Commit()? };
      Ok(result)
    })
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
}

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
  handle: isize,
  dcomp_device: IDCompositionDesktopDevice,
  _dcomp_target: IDCompositionTarget,
  dcomp_visual: IDCompositionVisual3,
  scale_transform: IDCompositionScaleTransform,
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
  /// Creates a transparent overlay HWND covering `outer_rect`, captures
  /// a screenshot of `window` via WGC, and orders the overlay just
  /// above the source window.
  pub(crate) fn new(
    context: &AnimationContext,
    window: &NativeWindow,
    inner_rect: &Rect,
    outer_rect: &Rect,
    opacity: Option<OpacityValue>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    COM_INIT.with(|_| {
      let captured = capture_window_texture(window.inner.hwnd(), context)?;

      let mut desc = D3D11_TEXTURE2D_DESC::default();
      unsafe { captured.texture.GetDesc(&raw mut desc) };

      let origin_x = outer_rect.x();
      let origin_y = outer_rect.y();
      let source_hwnd = window.inner.hwnd().0;

      let handle = dispatcher
        .dispatch_sync(|| create_window(source_hwnd, outer_rect))??;

      let dcomp_device = &context.dcomp_device;
      let dcomp_target =
        unsafe { dcomp_device.CreateTargetForHwnd(HWND(handle), true)? };

      let dcomp_surface = unsafe {
        dcomp_device.CreateSurface(
          desc.Width,
          desc.Height,
          DXGI_FORMAT_B8G8R8A8_UNORM,
          DXGI_ALPHA_MODE_PREMULTIPLIED,
        )?
      };

      copy_texture_to_surface(
        &context.d3d_context,
        &captured.texture,
        &dcomp_surface,
      )?;

      captured.close()?;

      let dcomp_visual: IDCompositionVisual3 =
        unsafe { dcomp_device.CreateVisual()?.cast()? };

      unsafe {
        dcomp_visual.SetContent(&dcomp_surface)?;
        dcomp_target.SetRoot(&dcomp_visual)?;
      }

      #[allow(clippy::cast_precision_loss)]
      unsafe {
        dcomp_visual.SetOffsetX2((inner_rect.x() - origin_x) as f32)?;
        dcomp_visual.SetOffsetY2((inner_rect.y() - origin_y) as f32)?;
      }

      let scale_transform =
        unsafe { dcomp_device.CreateScaleTransform()? };

      unsafe { dcomp_visual.SetTransform(&scale_transform)? };

      update_scale(
        &scale_transform,
        inner_rect.width(),
        inner_rect.height(),
        desc.Width,
        desc.Height,
      )?;

      if let Some(opacity) = opacity {
        unsafe { dcomp_visual.SetOpacity2(opacity.0)? };
      }

      unsafe { dcomp_device.Commit()? };

      Ok(Self {
        handle,
        dcomp_device: dcomp_device.clone(),
        _dcomp_target: dcomp_target,
        dcomp_visual,
        scale_transform,
        src_width: desc.Width,
        src_height: desc.Height,
        origin_x,
        origin_y,
        dispatcher: dispatcher.clone(),
      })
    })
  }

  /// Resizes the overlay HWND to cover `outer_rect`, updating the
  /// stored origin.
  ///
  /// Called when an animation's target changes mid-flight so the
  /// existing screenshot and z-order are preserved.
  pub(crate) fn resize(&mut self, outer_rect: &Rect) -> crate::Result<()> {
    self.origin_x = outer_rect.x();
    self.origin_y = outer_rect.y();

    unsafe {
      SetWindowPos(
        HWND(self.handle),
        None,
        outer_rect.x(),
        outer_rect.y(),
        outer_rect.width(),
        outer_rect.height(),
        SWP_NOACTIVATE,
      )
    }
    .map_err(crate::Error::from)
  }

  /// Repositions the DirectComposition visual within the overlay HWND
  /// and updates opacity.
  ///
  /// The HWND frame is never changed; only the visual moves. Must be
  /// called inside `AnimationContext::transaction`.
  pub(crate) fn update(
    &self,
    inner_rect: &Rect,
    opacity: Option<&OpacityValue>,
  ) -> crate::Result<()> {
    #[allow(clippy::cast_precision_loss)]
    unsafe {
      self
        .dcomp_visual
        .SetOffsetX2((inner_rect.x() - self.origin_x) as f32)?;
      self
        .dcomp_visual
        .SetOffsetY2((inner_rect.y() - self.origin_y) as f32)?;
    }

    update_scale(
      &self.scale_transform,
      inner_rect.width(),
      inner_rect.height(),
      self.src_width,
      self.src_height,
    )?;

    if let Some(opacity) = opacity {
      unsafe { self.dcomp_visual.SetOpacity2(opacity.0)? };
    }

    Ok(())
  }

  /// Tears down `DirectComposition` resources and destroys the overlay
  /// HWND.
  ///
  /// Clears the visual tree and commits before releasing COM objects so
  /// the compositor does not retain stale GPU surfaces.
  pub(crate) fn destroy(self) -> crate::Result<()> {
    COM_INIT.with(|_| {
      unsafe {
        if let Err(err) =
          self.dcomp_visual.SetContent(None::<&windows::core::IUnknown>)
        {
          tracing::warn!("Failed to clear DComp visual content: {err}");
        }
        if let Err(err) = self.dcomp_visual.SetTransform(
          None::<
            &windows::Win32::Graphics::DirectComposition::IDCompositionTransform,
          >,
        ) {
          tracing::warn!("Failed to clear DComp visual transform: {err}");
        }
        if let Err(err) = self.dcomp_device.Commit() {
          tracing::warn!("Failed to commit DComp teardown: {err}");
        }
      }

      self.dispatcher.dispatch_sync(|| {
        if let Err(err) = unsafe { DestroyWindow(HWND(self.handle)) } {
          tracing::warn!("Failed to destroy overlay HWND: {err}");
        }
      })
    })
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

/// Creates the overlay HWND for a single animation, sized to the given
/// rect and ordered just above `source_hwnd`.
fn create_window(source_hwnd: isize, rect: &Rect) -> crate::Result<isize> {
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
      rect.x(),
      rect.y(),
      rect.width(),
      rect.height(),
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

/// Captured WGC frame with its underlying texture.
///
/// Holds the frame, session, and pool alive until the texture has been
/// consumed (e.g. copied into a `DirectComposition` surface).
struct CapturedFrame {
  texture: ID3D11Texture2D,
  frame: Direct3D11CaptureFrame,
  session: GraphicsCaptureSession,
  frame_pool: Direct3D11CaptureFramePool,
}

impl CapturedFrame {
  /// Releases WGC resources in dependency order.
  fn close(self) -> crate::Result<()> {
    drop(self.texture);
    self.frame.Close()?;
    self.session.Close()?;
    self.frame_pool.Close()?;
    Ok(())
  }
}

/// Captures a single frame of a window via Windows.Graphics.Capture
/// and returns a `CapturedFrame` whose texture can be read until
/// `close()` is called.
///
/// Reuses the cached interop factory and WinRT device from
/// `AnimationContext`. A fresh frame pool is created per capture
/// because WGC pools cannot be reliably reused across sessions.
///
/// Perf: ~35-60ms. Single frame snapshots require waiting for DWM to
/// produce the next composed frame (~16.7ms at 60Hz).
fn capture_window_texture(
  hwnd: HWND,
  context: &AnimationContext,
) -> crate::Result<CapturedFrame> {
  // SAFETY: HWND is valid per the caller's contract.
  let capture_item: GraphicsCaptureItem =
    unsafe { context.capture_interop.CreateForWindow(hwnd)? };

  let item_size = capture_item.Size()?;

  let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
    &context.winrt_device,
    DirectXPixelFormat::B8G8R8A8UIntNormalized,
    1,
    item_size,
  )?;

  let session = frame_pool.CreateCaptureSession(&capture_item)?;

  // Prevent the cursor from being shown in the capture.
  let _ = session.SetIsCursorCaptureEnabled(false);

  // Disable the yellow capture border on Windows 11 (Build 22000+).
  let _ = session.SetIsBorderRequired(false);

  session.StartCapture()?;

  let frame = wait_for_frame(&frame_pool)?;

  let surface = frame.Surface()?;
  let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;

  // SAFETY: The WGC surface wraps a valid D3D11 texture.
  let texture: ID3D11Texture2D = unsafe { access.GetInterface()? };

  Ok(CapturedFrame {
    texture,
    frame,
    session,
    frame_pool,
  })
}

/// Waits for the next WGC frame using an event-based approach.
///
/// Registers a `FrameArrived` handler that signals a channel, then
/// blocks until the frame arrives or the timeout expires.
fn wait_for_frame(
  pool: &Direct3D11CaptureFramePool,
) -> crate::Result<Direct3D11CaptureFrame> {
  let (tx, rx) = std::sync::mpsc::sync_channel::<()>(1);

  pool.FrameArrived(&TypedEventHandler::new(
    move |_: &Option<Direct3D11CaptureFramePool>, _| {
      let _ = tx.send(());
      Ok(())
    },
  ))?;

  // Frame arrived before the handler was registered.
  if let Ok(frame) = pool.TryGetNextFrame() {
    return Ok(frame);
  }

  rx.recv_timeout(std::time::Duration::from_secs(1))
    .map_err(|_| {
      crate::Error::Platform("WGC capture timed out.".into())
    })?;

  pool.TryGetNextFrame().map_err(|err| {
    crate::Error::Platform(format!("Failed to get WGC frame: {err}"))
  })
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

/// Updates the scale factors on an existing `IDCompositionScaleTransform`
/// so the captured source is scaled to fill the target rect dimensions.
fn update_scale(
  transform: &IDCompositionScaleTransform,
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

  unsafe {
    transform.SetScaleX2(scale_x)?;
    transform.SetScaleY2(scale_y)?;
  }

  Ok(())
}
