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
        D3D11_SDK_VERSION,
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
      SetWindowPos, HTTRANSPARENT, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
      SWP_SHOWWINDOW, WM_NCHITTEST, WNDCLASSW, WS_EX_NOACTIVATE,
      WS_EX_NOREDIRECTIONBITMAP, WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{
  platform_impl::com::COM_INIT, Dispatcher, NativeWindow, OpacityValue,
  Rect, ThreadBound,
};

/// Guard ensuring the overlay window class is registered at most once.
static OVERLAY_CLASS: OnceLock<()> = OnceLock::new();

struct Direct3DDevice(IDirect3DDevice);
unsafe impl Send for Direct3DDevice {}

struct GraphicsCaptureItemInterop(IGraphicsCaptureItemInterop);
unsafe impl Send for GraphicsCaptureItemInterop {}

/// Shared GPU context for all animation windows.
///
/// Holds a single D3D11 device, `DirectComposition` device, and
/// pre-warmed WGC capture objects, avoiding the overhead of creating
/// heavyweight GPU/WGC objects per animation.
pub(crate) struct AnimationContext {
  _d3d_device: ID3D11Device,
  d3d_context: ThreadBound<ID3D11DeviceContext>,
  dcomp_device: ThreadBound<IDCompositionDesktopDevice>,
  winrt_device: Direct3DDevice,
  capture_interop: GraphicsCaptureItemInterop,
}

impl AnimationContext {
  /// Creates a shared D3D11 + `DirectComposition` device pair and
  /// pre-warms WGC capture objects (WinRT device, interop factory).
  pub(crate) fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    dispatcher.dispatch_sync(|| {
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
          d3d_context: ThreadBound::new(d3d_context, dispatcher.clone()),
          dcomp_device: ThreadBound::new(dcomp_device, dispatcher.clone()),
          winrt_device: Direct3DDevice(winrt_device),
          capture_interop: GraphicsCaptureItemInterop(capture_interop),
        })
      })
    })?
  }

  /// Executes `update_fn` and then commits all pending `DirectComposition`
  /// changes to the compositor in a single batch.
  pub(crate) fn transaction<F, R>(
    &self,
    update_fn: F,
    dispatcher: &Dispatcher,
  ) -> crate::Result<R>
  where
    F: FnOnce() -> R + Send,
    R: Send,
  {
    dispatcher.dispatch_sync(|| {
      COM_INIT.with(|_| {
        let result = update_fn();
        unsafe {
          self
            .dcomp_device
            .with(|dcomp_device| dcomp_device.Commit())??;
        }
        Ok(result)
      })
    })?
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

unsafe impl Sync for AnimationContext {}

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
  dcomp_device: ThreadBound<IDCompositionDesktopDevice>,
  _dcomp_target: ThreadBound<IDCompositionTarget>,
  dcomp_visual: ThreadBound<IDCompositionVisual3>,
  scale_transform: ThreadBound<IDCompositionScaleTransform>,
  /// Rect of the captured source texture (for scale calculations).
  src_inner_rect: Rect,
  /// Frame of the overlay HWND in screen coordinates.
  outer_rect: Rect,
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
    let captured = CapturedFrame::new(window.inner.hwnd(), context)?;

    dispatcher.dispatch_sync(|| {
      COM_INIT.with(|_| {
        // Window is spawned on the main thread - avoids having to create a
        // new message loop.
        let handle =
          Self::create_window(window.inner.hwnd().0, outer_rect)?;

        let dcomp_device = context.dcomp_device.get_ref()?;
        let dcomp_target =
          unsafe { dcomp_device.CreateTargetForHwnd(HWND(handle), true)? };

        let dcomp_surface = unsafe {
          dcomp_device.CreateSurface(
            inner_rect.width().cast_unsigned(),
            inner_rect.height().cast_unsigned(),
            DXGI_FORMAT_B8G8R8A8_UNORM,
            DXGI_ALPHA_MODE_PREMULTIPLIED,
          )?
        };

        Self::copy_texture_to_surface(
          context.d3d_context.get_ref()?,
          &captured.texture,
          &dcomp_surface,
        )?;

        let dcomp_visual: IDCompositionVisual3 =
          unsafe { dcomp_device.CreateVisual()?.cast()? };

        unsafe {
          dcomp_visual.SetContent(&dcomp_surface)?;
          dcomp_target.SetRoot(&dcomp_visual)?;
        }

        let scale_transform =
          unsafe { dcomp_device.CreateScaleTransform()? };

        unsafe { dcomp_visual.SetTransform(&scale_transform)? };

        let dcomp_visual =
          ThreadBound::new(dcomp_visual, dispatcher.clone());
        let scale_transform =
          ThreadBound::new(scale_transform, dispatcher.clone());
        let dcomp_target =
          ThreadBound::new(dcomp_target, dispatcher.clone());

        context.transaction(
          || {
            Self::update_visual(
              dcomp_visual.get_ref()?,
              scale_transform.get_ref()?,
              inner_rect,
              inner_rect,
              outer_rect,
              opacity.as_ref(),
            )
          },
          dispatcher,
        )??;

        Ok(Self {
          handle,
          dcomp_device: context.dcomp_device.clone(),
          _dcomp_target: dcomp_target,
          dcomp_visual,
          scale_transform,
          src_inner_rect: inner_rect.clone(),
          outer_rect: outer_rect.clone(),
          dispatcher: dispatcher.clone(),
        })
      })
    })?
  }

  /// Resizes the overlay HWND to cover `outer_rect`, updating the
  /// stored origin.
  ///
  /// Called when an animation's target changes mid-flight so the
  /// existing screenshot and z-order are preserved.
  pub(crate) fn resize(&mut self, outer_rect: &Rect) -> crate::Result<()> {
    self.outer_rect = outer_rect.clone();

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
    self.dispatcher.dispatch_sync(|| {
      Self::update_visual(
        self.dcomp_visual.get_ref()?,
        self.scale_transform.get_ref()?,
        &self.src_inner_rect,
        inner_rect,
        &self.outer_rect,
        opacity,
      )
    })?
  }

  /// Repositions the `DirectComposition` visual to `inner_rect` relative
  /// to `outer_rect`, scales it to fill the target dimensions, and
  /// optionally sets opacity.
  ///
  /// Shared by [`AnimationWindow::new`] and [`AnimationWindow::update`].
  /// Must be called inside `AnimationContext::transaction`.
  fn update_visual(
    dcomp_visual: &IDCompositionVisual3,
    scale_transform: &IDCompositionScaleTransform,
    src_inner_rect: &Rect,
    inner_rect: &Rect,
    outer_rect: &Rect,
    opacity: Option<&OpacityValue>,
  ) -> crate::Result<()> {
    #[allow(clippy::cast_precision_loss)]
    unsafe {
      dcomp_visual
        .SetOffsetX2((inner_rect.x() - outer_rect.x()) as f32)?;
      dcomp_visual
        .SetOffsetY2((inner_rect.y() - outer_rect.y()) as f32)?;
    }

    #[allow(clippy::cast_precision_loss)]
    let scale_x =
      inner_rect.width() as f32 / src_inner_rect.width() as f32;
    #[allow(clippy::cast_precision_loss)]
    let scale_y =
      inner_rect.height() as f32 / src_inner_rect.height() as f32;

    unsafe {
      scale_transform.SetScaleX2(scale_x)?;
      scale_transform.SetScaleY2(scale_y)?;
    }

    if let Some(opacity) = opacity {
      unsafe { dcomp_visual.SetOpacity2(opacity.0)? };
    }

    Ok(())
  }

  /// Tears down `DirectComposition` resources and destroys the overlay
  /// HWND.
  ///
  /// Clears the visual tree and commits before releasing COM objects so
  /// the compositor does not retain stale GPU surfaces.
  pub(crate) fn destroy(self) -> crate::Result<()> {
    self.dispatcher.dispatch_sync(|| {
      COM_INIT.with(|_| {
        unsafe {
          // Clear the surface reference so the compositor releases the GPU
          // resource immediately on the next commit.
          if let Err(err) = self.dcomp_visual.with(|dcomp_visual| {
            dcomp_visual.SetContent(None::<&windows::core::IUnknown>)
          }) {
            tracing::warn!("Failed to clear DComp visual content: {err}");
          }

          if let Err(err) =
            self.dcomp_device.with(|dcomp_device| dcomp_device.Commit())
          {
            tracing::warn!("Failed to commit DComp teardown: {err}");
          }

          if let Err(err) = DestroyWindow(HWND(self.handle)) {
            tracing::warn!("Failed to destroy overlay HWND: {err}");
          }
        }
      });
    })
  }

  /// Creates the overlay HWND for a single animation, sized to the given
  /// rect and ordered just above `source_hwnd`.
  fn create_window(
    source_hwnd: isize,
    rect: &Rect,
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

  /// Window procedure for the overlay class.
  unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    // Route all mouse inputs to the window below.
    if msg == WM_NCHITTEST {
      return LRESULT(HTTRANSPARENT as isize);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
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
  /// Captures a single frame of `hwnd` via Windows.Graphics.Capture.
  ///
  /// Reuses the cached interop factory and WinRT device from
  /// `AnimationContext`. A fresh frame pool is created per capture
  /// because WGC pools cannot be reliably reused across sessions.
  ///
  /// Perf: ~35-60ms. Single-frame captures require waiting for DWM to
  /// produce the next composed frame (i.e. up to 16.7ms at 60Hz).
  fn new(hwnd: HWND, context: &AnimationContext) -> crate::Result<Self> {
    // SAFETY: HWND is valid per the caller's contract.
    let capture_item: GraphicsCaptureItem =
      unsafe { context.capture_interop.0.CreateForWindow(hwnd)? };

    let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
      &context.winrt_device.0,
      DirectXPixelFormat::B8G8R8A8UIntNormalized,
      1,
      capture_item.Size()?,
    )?;

    let session = frame_pool.CreateCaptureSession(&capture_item)?;

    // Prevent the cursor from being shown in the capture.
    let _ = session.SetIsCursorCaptureEnabled(false);

    // Disable the yellow capture border on Windows 11 (Build 22000+).
    let _ = session.SetIsBorderRequired(false);

    let (tx, rx) = std::sync::mpsc::sync_channel::<()>(1);

    frame_pool.FrameArrived(&TypedEventHandler::new(
      move |_: &Option<Direct3D11CaptureFramePool>, _| {
        let _ = tx.send(());
        Ok(())
      },
    ))?;

    session.StartCapture()?;

    rx.recv_timeout(std::time::Duration::from_secs(1))
      .map_err(|_| {
        crate::Error::Platform("WGC capture timed out.".into())
      })?;

    let frame = frame_pool.TryGetNextFrame().map_err(|err| {
      crate::Error::Platform(format!("Failed to get WGC frame: {err}"))
    })?;

    let surface = frame.Surface()?;
    let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;

    // SAFETY: The WGC surface wraps a valid D3D11 texture.
    let texture: ID3D11Texture2D = unsafe { access.GetInterface()? };

    Ok(Self {
      texture,
      frame,
      session,
      frame_pool,
    })
  }

  /// Releases WGC resources in dependency order.
  fn close(&self) -> crate::Result<()> {
    self.frame.Close()?;
    self.session.Close()?;
    self.frame_pool.Close()?;
    Ok(())
  }
}

impl Drop for CapturedFrame {
  fn drop(&mut self) {
    let _ = self.close();
  }
}
