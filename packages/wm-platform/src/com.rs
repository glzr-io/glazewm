use anyhow::Context;
use windows::{
  core::{ComInterface, IUnknown, IUnknown_Vtbl, GUID, HRESULT},
  Win32::{
    System::Com::{
      CoCreateInstance, CoInitializeEx, CoUninitialize, IServiceProvider,
      CLSCTX_ALL, CLSCTX_SERVER, COINIT_APARTMENTTHREADED,
    },
    UI::Shell::{ITaskbarList2, TaskbarList},
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

pub struct ComInit {
  service_provider: Option<IServiceProvider>,
  application_view_collection: Option<IApplicationViewCollection>,
  taskbar_list: Option<ITaskbarList2>,
}

impl ComInit {
  /// Initializes COM on the current thread with apartment threading model.
  /// `COINIT_APARTMENTTHREADED` is required for shell COM objects.
  ///
  /// # Panics
  ///
  /// Panics if COM initialization fails. This is typically only possible
  /// if COM is already initialized with an incompatible threading model.
  #[must_use]
  pub fn new() -> Self {
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }
      .expect("Unable to initialize COM.");

    let service_provider = unsafe {
      CoCreateInstance(&CLSID_IMMERSIVE_SHELL, None, CLSCTX_ALL)
    }
    .ok();

    let application_view_collection = service_provider.as_ref().and_then(
      |provider: &IServiceProvider| unsafe {
        provider.QueryService(&IApplicationViewCollection::IID).ok()
      },
    );

    let taskbar_list =
      unsafe { CoCreateInstance(&TaskbarList, None, CLSCTX_SERVER) }.ok();

    Self {
      service_provider,
      application_view_collection,
      taskbar_list,
    }
  }

  /// Returns an instance of `IServiceProvider`.
  pub fn service_provider(&self) -> anyhow::Result<IServiceProvider> {
    self
      .service_provider
      .clone()
      .context("Unable to create `IServiceProvider` instance.")
  }

  /// Returns an instance of `IApplicationViewCollection`.
  pub fn application_view_collection(
    &self,
  ) -> anyhow::Result<IApplicationViewCollection> {
    self.application_view_collection.clone().context(
      "Failed to query for `IApplicationViewCollection` instance.",
    )
  }

  /// Returns an instance of `ITaskbarList2`.
  pub fn taskbar_list(&self) -> anyhow::Result<ITaskbarList2> {
    self
      .taskbar_list
      .clone()
      .context("Unable to create `ITaskbarList2` instance.")
  }
}

impl Default for ComInit {
  fn default() -> Self {
    Self::new()
  }
}

impl Drop for ComInit {
  fn drop(&mut self) {
    // Explicitly drop COM interfaces first.
    drop(self.taskbar_list.take());
    drop(self.application_view_collection.take());
    drop(self.service_provider.take());

    unsafe { CoUninitialize() };
  }
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
