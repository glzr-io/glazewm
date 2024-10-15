// Cloaking functionality taken from https://github.com/Ciantic/AltTabAccessor/blob/main/src/lib.rs
use std::ffi::c_void;
use windows::core::{ComInterface, Interface};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED};
use crate::common::com::interfaces::{IApplicationViewCollection, IServiceProvider, CLSID_IMMERSIVE_SHELL};

mod interfaces;

pub enum CloakVisibility {
    HIDDEN,
    VISIBLE
}

struct ComInit();

impl ComInit {
    pub fn new() -> Self {
        unsafe {
            // Apparently only COINIT_APARTMENTTHREADED works correctly.
            // Initialize the COM Library
            // Handle the error differently maybe?
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
        }
        Self()
    }
}

impl Drop for ComInit {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}


fn get_iservice_provider() -> IServiceProvider {
    COM_INIT.with(|_| unsafe {
        CoCreateInstance(&CLSID_IMMERSIVE_SHELL, None, CLSCTX_ALL).unwrap()
    })
}

fn get_iapplication_view_collection(provider: &IServiceProvider) -> IApplicationViewCollection {
    COM_INIT.with(|_| {
        let mut obj = std::ptr::null_mut::<c_void>();
        unsafe {
            provider
                .query_service(
                    &IApplicationViewCollection::IID,
                    &IApplicationViewCollection::IID,
                    &mut obj,
                )
                .unwrap();
        }

        assert!(!obj.is_null());

        unsafe { IApplicationViewCollection::from_raw(obj) }
    })
}

// Each thread that accesses the COM_INIT variable gets a local instance of the variable.
// This is needed since the COM library requires the CoInitializeEx needs to be initialized per thread.
thread_local! {
    static COM_INIT: ComInit = ComInit::new();
}

pub fn set_cloak(hwnd: HWND, cloak_visibility: &CloakVisibility) {
    COM_INIT.with(|_| {
        let provider = get_iservice_provider();
        let view_collection = get_iapplication_view_collection(&provider);
        let mut view = None;
        unsafe {
            view_collection.get_view_for_hwnd(hwnd, &mut view).unwrap()
        };
        let view = view.unwrap();


        unsafe {
            // https://github.com/Ciantic/AltTabAccessor/issues/1#issuecomment-1426877843
            let flag = match cloak_visibility {
                CloakVisibility::VISIBLE => 0,
                CloakVisibility::HIDDEN => 2
            };
            view.set_cloak(1, flag).unwrap()
        }
    });

}
