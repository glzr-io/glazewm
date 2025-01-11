use anyhow::Context;
use windows::{
  core::{ComInterface, IUnknown, IUnknown_Vtbl, GUID, HRESULT},
  Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IServiceProvider,
    CLSCTX_ALL, COINIT_APARTMENTTHREADED,
  },
};

/// COM class identifier (CLSID) for the Windows Shell that implements the
/// `IServiceProvider` interface.
pub const CLSID_IMMERSIVE_SHELL: GUID =
  GUID::from_u128(0xC2F03A33_21F5_47FA_B4BB_156362A2F239);

thread_local! {
  /// Manages per-thread COM initialization. COM must be initialized on each
  /// thread that uses it, so we store this in thread-local storage to handle
  /// the setup and cleanup automatically.
  pub static COM_INIT: ComInit = ComInit::new();
}

pub struct ComInit();

impl ComInit {
  /// Initializes COM on the current thread with apartment threading model.
  /// `COINIT_APARTMENTTHREADED` is required for shell COM objects.
  ///
  /// # Panics
  ///
  /// Panics if COM initialization fails. This is typically only possible
  /// if COM is already initialized with an incompatible threading model.
  pub fn new() -> Self {
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
      .expect("Unable to initialize COM.");

    Self()
  }
}

impl Drop for ComInit {
  fn drop(&mut self) {
    unsafe { CoUninitialize() };
  }
}

/// Returns an instance of `IServiceProvider`.
pub fn iservice_provider() -> anyhow::Result<IServiceProvider> {
  COM_INIT.with(|_| unsafe {
    CoCreateInstance(&CLSID_IMMERSIVE_SHELL, None, CLSCTX_ALL)
      .context("Unable to create `IServiceProvider` instance.")
  })
}

/// Returns an instance of `IApplicationViewCollection`.
pub fn iapplication_view_collection(
  provider: &IServiceProvider,
) -> anyhow::Result<IApplicationViewCollection> {
  COM_INIT.with(|_| {
    unsafe { provider.QueryService(&IApplicationViewCollection::IID) }
      .context(
        "Failed to query for `IApplicationViewCollection` instance.",
      )
  })
}

/// Undocumented COM interface for Windows shell functionality.
///
/// Note that filler methods are added to match the vtable layout.
#[windows_interface::interface("1841c6d7-4f9d-42c0-af41-8747538f10e5")]
pub unsafe trait IApplicationViewCollection: IUnknown {
  pub unsafe fn m1(&self);
  pub unsafe fn m2(&self);
  pub unsafe fn m3(&self);
  pub unsafe fn get_view_for_hwnd(
    &self,
    window: isize,
    application_view: *mut Option<IApplicationView>,
  ) -> HRESULT;
}

/// Undocumented COM interface for managing views in the Windows shell.
///
/// Note that filler methods are added to match the vtable layout.
#[windows_interface::interface("372E1D3B-38D3-42E4-A15B-8AB2B178F513")]
pub unsafe trait IApplicationView: IUnknown {
  pub unsafe fn m1(&self);
  pub unsafe fn m2(&self);
  pub unsafe fn m3(&self);
  pub unsafe fn m4(&self);
  pub unsafe fn m5(&self);
  pub unsafe fn m6(&self);
  pub unsafe fn m7(&self);
  pub unsafe fn m8(&self);
  pub unsafe fn m9(&self);
  pub unsafe fn set_cloak(
    &self,
    cloak_type: u32,
    cloak_flag: i32,
  ) -> HRESULT;
}
