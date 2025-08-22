# WM-Platform Display API - Comprehensive Implementation Plan

## Executive Summary

This document provides a detailed implementation plan for creating a production-ready, comprehensive display API for the wm-platform crate. The plan addresses the numerous TODOs, missing implementations, and compilation errors found in the current codebase while designing a robust, cross-platform display management system.

## Current State Analysis

### Issues Identified

1. **Missing Method Implementations (Windows)**:
   - `get_monitor_info()` - Called but not implemented
   - `get_device_name()` - Called but not implemented  
   - `get_state_flags()` - Called but not implemented
   - `output_technology()` - Returns `todo!()`
   - `is_builtin()` - Returns `todo!()`

2. **Missing Method Implementations (macOS)**:
   - `id()` for Display - Returns `todo!()`
   - `is_builtin()` for DisplayExtMacos - Returns `todo!()`
   - `all_display_devices()` - Returns `todo!()`
   - `active_display_devices()` - Returns `todo!()`
   - `display_from_point()` - Returns `todo!()`
   - `primary_display()` - Returns `todo!()`

3. **Missing Error Types**:
   - `DisplayNotFound` - Used but not defined in error.rs
   - Various error variants for display operations

4. **Missing Public API Functions**:
   - No top-level functions exposed for display enumeration
   - Missing convenience functions for common operations

5. **Incomplete Trait Implementations**:
   - Missing `PartialEq` and `Eq` for Display and DisplayDevice
   - Missing comprehensive error handling

## Architecture Design

### Display API Surface

The comprehensive display API will provide these main entry points:

```rust
// Top-level convenience functions
pub fn all_displays() -> Result<Vec<Display>>;
pub fn all_display_devices() -> Result<Vec<DisplayDevice>>;  
pub fn primary_display() -> Result<Display>;
pub fn display_from_point(point: Point) -> Result<Display>;

// Display methods
impl Display {
    pub fn id(&self) -> DisplayId;
    pub fn name(&self) -> Result<String>;
    pub fn bounds(&self) -> Result<Rect>;
    pub fn working_area(&self) -> Result<Rect>;
    pub fn scale_factor(&self) -> Result<f32>;
    pub fn dpi(&self) -> Result<u32>;
    pub fn refresh_rate(&self) -> Result<f32>;
    pub fn is_primary(&self) -> Result<bool>;
    pub fn devices(&self) -> Result<Vec<DisplayDevice>>;
    pub fn main_device(&self) -> Result<Option<DisplayDevice>>;
    pub fn color_space(&self) -> Result<String>;
    pub fn bit_depth(&self) -> Result<u32>;
    pub fn rotation(&self) -> Result<f32>;
}

// DisplayDevice methods
impl DisplayDevice {
    pub fn id(&self) -> DisplayDeviceId;
    pub fn name(&self) -> Result<String>;
    pub fn description(&self) -> Result<String>;
    pub fn is_builtin(&self) -> Result<bool>;
    pub fn output_technology(&self) -> Result<OutputTechnology>;
    pub fn connection_state(&self) -> Result<ConnectionState>;
    pub fn mirroring_state(&self) -> Result<Option<MirroringState>>;
    pub fn refresh_rate(&self) -> Result<f32>;
    pub fn rotation(&self) -> Result<f32>;
    pub fn supported_modes(&self) -> Result<Vec<DisplayMode>>;
    pub fn current_mode(&self) -> Result<DisplayMode>;
}
```

### Error Handling Strategy

Comprehensive error types using thiserror:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DisplayError {
    #[error("Display not found")]
    DisplayNotFound,
    
    #[error("Primary display not found")]
    PrimaryDisplayNotFound,
    
    #[error("Display device not found")]
    DisplayDeviceNotFound,
    
    #[error("Display mode not found")]
    DisplayModeNotFound,
    
    #[error("Display enumeration failed: {0}")]
    DisplayEnumerationFailed(String),
    
    #[error("Display configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Display driver error: {0}")]
    DriverError(String),
    
    #[error("Insufficient permissions for display operation")]
    PermissionDenied,
    
    #[cfg(target_os = "windows")]
    #[error("Windows display API error: {0}")]
    WindowsApiError(#[from] windows::core::Error),
    
    #[cfg(target_os = "macos")]
    #[error("Core Graphics error: {code}")]
    CoreGraphicsError { code: i32 },
}
```

## Detailed Implementation Plan

### File-by-File Changes

#### 1. `/packages/wm-platform/src/error.rs`

**Changes Required:**
- Add missing display-related error variants
- Improve error messages with context
- Add platform-specific error mappings

**Implementation:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // Existing errors...
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    #[cfg(target_os = "windows")]
    Windows(#[from] windows::core::Error),

    // New display-related errors
    #[error("Display not found")]
    DisplayNotFound,

    #[error("Primary display not found")]
    PrimaryDisplayNotFound,

    #[error("Display device not found")]
    DisplayDeviceNotFound,

    #[error("Display mode not found")]
    DisplayModeNotFound,

    #[error("Display enumeration failed: {0}")]
    DisplayEnumerationFailed(String),

    #[error("Display configuration error: {0}")]
    DisplayConfigurationError(String),

    #[error("Display driver error: {0}")]
    DisplayDriverError(String),

    #[error("Insufficient permissions for display operation")]
    DisplayPermissionDenied,

    // Platform-specific display errors
    #[cfg(target_os = "windows")]
    #[error("Monitor handle invalid")]
    InvalidMonitorHandle,

    #[cfg(target_os = "macos")]
    #[error("Core Graphics display operation failed: {code}")]
    CoreGraphicsError { code: i32 },
}
```

#### 2. `/packages/wm-platform/src/display.rs`

**Changes Required:**
- Add missing public API functions
- Implement PartialEq/Eq for Display and DisplayDevice
- Add new display properties and methods
- Add comprehensive rustdoc documentation

**New Additions:**
```rust
/// Display mode information.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: f32,
    pub bit_depth: u32,
    pub is_interlaced: bool,
}

/// Enhanced OutputTechnology enum
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputTechnology {
    Internal,
    VGA,
    DVI,
    DVIDigital,
    DVIAnalog,
    HDMI,
    DisplayPort,
    Thunderbolt,
    Thunderbolt2,
    Thunderbolt3,
    Thunderbolt4,
    USB,
    USBTypeC,
    Wireless,
    MiniDisplayPort,
    MiniDVI,
    Composite,
    SVideo,
    Component,
    Unknown,
}

// Top-level convenience functions
/// Gets all active displays on the system.
pub fn all_displays() -> Result<Vec<Display>> {
    #[cfg(target_os = "windows")]
    return platform_impl::all_displays().map(|displays| {
        displays.into_iter().map(Display::from_platform_impl).collect()
    });
    
    #[cfg(target_os = "macos")]
    {
        use crate::platform_impl::EventLoopDispatcher;
        let dispatcher = EventLoopDispatcher::current()?;
        platform_impl::all_displays(dispatcher).map(|displays| {
            displays.into_iter().map(Display::from_platform_impl).collect()
        })
    }
}

/// Gets all display devices on the system.
pub fn all_display_devices() -> Result<Vec<DisplayDevice>> {
    #[cfg(target_os = "windows")]
    return platform_impl::all_display_devices().map(|devices| {
        devices.into_iter().map(DisplayDevice::from_platform_impl).collect()
    });
    
    #[cfg(target_os = "macos")]
    {
        use crate::platform_impl::EventLoopDispatcher;
        let dispatcher = EventLoopDispatcher::current()?;
        platform_impl::all_display_devices(dispatcher).map(|devices| {
            devices.into_iter().map(DisplayDevice::from_platform_impl).collect()
        })
    }
}

/// Gets the primary display.
pub fn primary_display() -> Result<Display> {
    #[cfg(target_os = "windows")]
    return platform_impl::primary_display().map(Display::from_platform_impl);
    
    #[cfg(target_os = "macos")]
    {
        use crate::platform_impl::EventLoopDispatcher;
        let dispatcher = EventLoopDispatcher::current()?;
        platform_impl::primary_display(dispatcher).map(Display::from_platform_impl)
    }
}

/// Gets the display containing the given point.
pub fn display_from_point(point: Point) -> Result<Display> {
    #[cfg(target_os = "windows")]
    return platform_impl::display_from_point(point).map(Display::from_platform_impl);
    
    #[cfg(target_os = "macos")]
    {
        use crate::platform_impl::EventLoopDispatcher;
        let dispatcher = EventLoopDispatcher::current()?;
        platform_impl::display_from_point(point, dispatcher).map(Display::from_platform_impl)
    }
}

// Implement PartialEq/Eq for Display and DisplayDevice
impl PartialEq for Display {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Display {}

impl PartialEq for DisplayDevice {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for DisplayDevice {}

impl Display {
    /// Gets the display's refresh rate in Hz.
    pub fn refresh_rate(&self) -> Result<f32> {
        self.inner.refresh_rate()
    }

    /// Gets the display's color space.
    pub fn color_space(&self) -> Result<String> {
        self.inner.color_space()
    }

    /// Gets the display's bit depth.
    pub fn bit_depth(&self) -> Result<u32> {
        self.inner.bit_depth()
    }

    /// Gets the display's rotation in degrees.
    pub fn rotation(&self) -> Result<f32> {
        self.inner.rotation()
    }
}

impl DisplayDevice {
    /// Gets the display device's human-readable name.
    pub fn name(&self) -> Result<String> {
        self.inner.name()
    }

    /// Gets the display device's description.
    pub fn description(&self) -> Result<String> {
        self.inner.description()
    }

    /// Gets the supported display modes for this device.
    pub fn supported_modes(&self) -> Result<Vec<DisplayMode>> {
        self.inner.supported_modes()
    }

    /// Gets the current display mode for this device.
    pub fn current_mode(&self) -> Result<DisplayMode> {
        self.inner.current_mode()
    }
}
```

#### 3. `/packages/wm-platform/src/platform_impl/windows/display.rs`

**Changes Required:**
- Implement all missing methods
- Fix compilation errors
- Add comprehensive Windows display API integration
- Implement proper error handling
- Add support for new display properties

**Key Missing Method Implementations:**

```rust
impl Display {
    /// Gets the monitor info structure from Windows API.
    fn get_monitor_info(&self) -> Result<MONITORINFOEXW> {
        let mut monitor_info = MONITORINFOEXW {
            monitorInfo: MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFOEXW>().try_into()
                    .map_err(|_| Error::DisplayConfigurationError("Invalid monitor info size".to_string()))?,
                ..Default::default()
            },
            ..Default::default()
        };

        unsafe {
            GetMonitorInfoW(
                HMONITOR(self.monitor_handle),
                std::ptr::from_mut(&mut monitor_info).cast(),
            )
        }
        .map_err(|e| Error::Windows(e))?;

        Ok(monitor_info)
    }

    /// Gets the device name for this display.
    fn get_device_name(&self) -> Result<String> {
        let monitor_info = self.get_monitor_info()?;
        Ok(
            String::from_utf16_lossy(&monitor_info.szDevice)
                .trim_end_matches('\0')
                .to_string(),
        )
    }

    /// Gets the display's refresh rate.
    pub fn refresh_rate(&self) -> Result<f32> {
        let device_name = self.get_device_name()?;
        let device_mode = self.get_current_device_mode(&device_name)?;
        Ok(device_mode.dmDisplayFrequency as f32)
    }

    /// Gets the display's color space.
    pub fn color_space(&self) -> Result<String> {
        // Windows doesn't easily expose color space info via basic APIs
        // This would require more advanced color management APIs
        Ok("sRGB".to_string()) // Default assumption
    }

    /// Gets the display's bit depth.
    pub fn bit_depth(&self) -> Result<u32> {
        let device_name = self.get_device_name()?;
        let device_mode = self.get_current_device_mode(&device_name)?;
        Ok(device_mode.dmBitsPerPel)
    }

    /// Gets the display's rotation.
    pub fn rotation(&self) -> Result<f32> {
        let device_name = self.get_device_name()?;
        let device_mode = self.get_current_device_mode(&device_name)?;
        
        Ok(match device_mode.dmDisplayOrientation {
            0 => 0.0,   // DMDO_DEFAULT
            1 => 90.0,  // DMDO_90
            2 => 180.0, // DMDO_180
            3 => 270.0, // DMDO_270
            _ => 0.0,
        })
    }

    /// Gets the current device mode for the display.
    fn get_current_device_mode(&self, device_name: &str) -> Result<DEVMODEW> {
        let mut device_mode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>().try_into()
                .map_err(|_| Error::DisplayConfigurationError("Invalid device mode size".to_string()))?,
            ..Default::default()
        };

        let device_name_wide: Vec<u16> = device_name.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            EnumDisplaySettingsW(
                PCWSTR(device_name_wide.as_ptr()),
                u32::MAX, // ENUM_CURRENT_SETTINGS
                &raw mut device_mode,
            )
        }
        .map_err(|e| Error::Windows(e))?;

        Ok(device_mode)
    }
}

impl DisplayDevice {
    /// Gets the device's human-readable name.
    pub fn name(&self) -> Result<String> {
        self.get_device_string()
    }

    /// Gets the device's description.
    pub fn description(&self) -> Result<String> {
        self.get_device_string()
    }

    /// Gets the output technology.
    pub fn output_technology(&self) -> Result<OutputTechnology> {
        // Windows API doesn't directly expose this information easily
        // Would need to use Setup API or PnP API for detailed device info
        // For now, return based on common patterns in device names/IDs
        let device_id = &self.hardware_id;
        
        if device_id.contains("Internal") || device_id.contains("Laptop") {
            Ok(OutputTechnology::Internal)
        } else if device_id.contains("HDMI") {
            Ok(OutputTechnology::HDMI)
        } else if device_id.contains("DisplayPort") || device_id.contains("DP") {
            Ok(OutputTechnology::DisplayPort)
        } else if device_id.contains("DVI") {
            Ok(OutputTechnology::DVI)
        } else if device_id.contains("VGA") {
            Ok(OutputTechnology::VGA)
        } else if device_id.contains("USB") {
            Ok(OutputTechnology::USB)
        } else {
            Ok(OutputTechnology::Unknown)
        }
    }

    /// Returns whether this is a built-in device.
    pub fn is_builtin(&self) -> Result<bool> {
        // Check device ID patterns that indicate built-in displays
        let device_id = &self.hardware_id;
        Ok(device_id.contains("Internal") || 
           device_id.contains("Laptop") ||
           device_id.contains("Built") ||
           device_id.contains("Embedded"))
    }

    /// Gets the state flags from Windows API.
    fn get_state_flags(&self) -> Result<u32> {
        let mut display_device = DISPLAY_DEVICEW {
            cb: std::mem::size_of::<DISPLAY_DEVICEW>().try_into()
                .map_err(|_| Error::DisplayConfigurationError("Invalid display device size".to_string()))?,
            ..Default::default()
        };

        // Find the device by device name
        let mut device_index = 0u32;
        loop {
            let result = unsafe {
                EnumDisplayDevicesW(
                    PCWSTR::null(),
                    device_index,
                    &raw mut display_device,
                    0,
                )
            };

            if !result.as_bool() {
                break;
            }

            let current_device_name = String::from_utf16_lossy(&display_device.DeviceName)
                .trim_end_matches('\0')
                .to_string();

            if current_device_name == self.device_name {
                return Ok(display_device.StateFlags);
            }

            device_index += 1;
        }

        Err(Error::DisplayDeviceNotFound)
    }

    /// Gets supported display modes for this device.
    pub fn supported_modes(&self) -> Result<Vec<DisplayMode>> {
        let mut modes = Vec::new();
        let mut mode_index = 0u32;
        let device_name_wide: Vec<u16> = self.device_name.encode_utf16().chain(std::iter::once(0)).collect();

        loop {
            let mut device_mode = DEVMODEW {
                dmSize: std::mem::size_of::<DEVMODEW>().try_into()
                    .map_err(|_| Error::DisplayConfigurationError("Invalid device mode size".to_string()))?,
                ..Default::default()
            };

            let result = unsafe {
                EnumDisplaySettingsW(
                    PCWSTR(device_name_wide.as_ptr()),
                    mode_index,
                    &raw mut device_mode,
                )
            };

            if !result.as_bool() {
                break;
            }

            modes.push(DisplayMode {
                width: device_mode.dmPelsWidth,
                height: device_mode.dmPelsHeight,
                refresh_rate: device_mode.dmDisplayFrequency as f32,
                bit_depth: device_mode.dmBitsPerPel,
                is_interlaced: (device_mode.dmDisplayFlags & 0x2) != 0, // DM_INTERLACED
            });

            mode_index += 1;
        }

        Ok(modes)
    }

    /// Gets the current display mode for this device.
    pub fn current_mode(&self) -> Result<DisplayMode> {
        let device_name_wide: Vec<u16> = self.device_name.encode_utf16().chain(std::iter::once(0)).collect();
        
        let mut device_mode = DEVMODEW {
            dmSize: std::mem::size_of::<DEVMODEW>().try_into()
                .map_err(|_| Error::DisplayConfigurationError("Invalid device mode size".to_string()))?,
            ..Default::default()
        };

        unsafe {
            EnumDisplaySettingsW(
                PCWSTR(device_name_wide.as_ptr()),
                u32::MAX, // ENUM_CURRENT_SETTINGS
                &raw mut device_mode,
            )
        }
        .map_err(|e| Error::Windows(e))?;

        Ok(DisplayMode {
            width: device_mode.dmPelsWidth,
            height: device_mode.dmPelsHeight,
            refresh_rate: device_mode.dmDisplayFrequency as f32,
            bit_depth: device_mode.dmBitsPerPel,
            is_interlaced: (device_mode.dmDisplayFlags & 0x2) != 0, // DM_INTERLACED
        })
    }
}
```

#### 4. `/packages/wm-platform/src/platform_impl/macos/display.rs`

**Changes Required:**
- Implement all missing methods marked with `todo!()`
- Fix compilation errors
- Add comprehensive macOS display API integration using Core Graphics
- Implement proper thread-safe access patterns

**Key Missing Method Implementations:**

```rust
impl Display {
    /// Gets the Display ID.
    pub fn id(&self) -> DisplayId {
        // Use a combination of CG display ID and screen info for unique identification
        // Since we need a stable ID across app restarts
        DisplayId(self.cg_display_id)
    }

    /// Gets the display's refresh rate.
    pub fn refresh_rate(&self) -> Result<f32> {
        let display_mode = unsafe { CGDisplayCopyDisplayMode(self.cg_display_id) }
            .ok_or(Error::DisplayModeNotFound)?;

        let refresh_rate = unsafe { CGDisplayMode::refresh_rate(Some(&display_mode)) };
        Ok(refresh_rate as f32)
    }

    /// Gets the display's color space.
    pub fn color_space(&self) -> Result<String> {
        // macOS Core Graphics API for color space detection
        use objc2_core_graphics::{CGDisplayCopyColorSpace, CGColorSpaceGetName};
        
        let color_space = unsafe { CGDisplayCopyColorSpace(self.cg_display_id) }
            .ok_or(Error::DisplayConfigurationError("Could not get color space".to_string()))?;
        
        let name = unsafe { CGColorSpaceGetName(color_space.as_ref()) };
        // Convert CFStringRef to Rust String
        // This would need proper CFString handling
        Ok("sRGB IEC61966-2.1".to_string()) // Default assumption
    }

    /// Gets the display's bit depth.
    pub fn bit_depth(&self) -> Result<u32> {
        let display_mode = unsafe { CGDisplayCopyDisplayMode(self.cg_display_id) }
            .ok_or(Error::DisplayModeNotFound)?;

        // Core Graphics doesn't easily expose bit depth, estimate from pixel format
        Ok(32) // Most modern displays are 32-bit
    }

    /// Gets the display's rotation.
    pub fn rotation(&self) -> Result<f32> {
        // Use Core Graphics Display Services API
        use objc2_core_graphics::CGDisplayRotation;
        
        let rotation = unsafe { CGDisplayRotation(self.cg_display_id) };
        Ok(rotation as f32)
    }
}

impl DisplayExtMacos for Display {
    fn is_builtin(&self) -> Result<bool> {
        use objc2_core_graphics::CGDisplayIsBuiltin;
        
        let is_builtin = unsafe { CGDisplayIsBuiltin(self.cg_display_id) };
        Ok(is_builtin != 0)
    }
}

impl DisplayDevice {
    /// Gets the display device's human-readable name.
    pub fn name(&self) -> Result<String> {
        // Use Core Graphics to get display product name
        use objc2_core_graphics::CGDisplayIOServicePort;
        use objc2_core_foundation::{CFDictionary, kCFStringEncodingUTF8};
        
        // This is a simplified implementation - real implementation would need
        // proper IOKit integration to get device names
        Ok(format!("Display Device {}", self.cg_display_id))
    }

    /// Gets the display device's description.
    pub fn description(&self) -> Result<String> {
        self.name() // For now, same as name
    }

    /// Gets supported display modes for this device.
    pub fn supported_modes(&self) -> Result<Vec<DisplayMode>> {
        use objc2_core_graphics::{CGDisplayCopyAllDisplayModes, CGDisplayMode};
        
        let modes_array = unsafe { CGDisplayCopyAllDisplayModes(self.cg_display_id, std::ptr::null()) }
            .ok_or(Error::DisplayConfigurationError("Could not get display modes".to_string()))?;

        let mut modes = Vec::new();
        let count = unsafe { CFArrayGetCount(modes_array.as_ref()) };
        
        for i in 0..count {
            let mode = unsafe { CFArrayGetValueAtIndex(modes_array.as_ref(), i) };
            // Cast to CGDisplayModeRef and extract properties
            // This would need proper Core Foundation array handling
            
            modes.push(DisplayMode {
                width: 1920, // Placeholder - would extract from mode
                height: 1080,
                refresh_rate: 60.0,
                bit_depth: 32,
                is_interlaced: false,
            });
        }

        Ok(modes)
    }

    /// Gets the current display mode for this device.
    pub fn current_mode(&self) -> Result<DisplayMode> {
        let display_mode = unsafe { CGDisplayCopyDisplayMode(self.cg_display_id) }
            .ok_or(Error::DisplayModeNotFound)?;

        let width = unsafe { CGDisplayMode::width(&display_mode) };
        let height = unsafe { CGDisplayMode::height(&display_mode) };
        let refresh_rate = unsafe { CGDisplayMode::refresh_rate(Some(&display_mode)) };

        Ok(DisplayMode {
            width: width as u32,
            height: height as u32,
            refresh_rate: refresh_rate as f32,
            bit_depth: 32, // Default assumption
            is_interlaced: false, // Modern displays typically aren't interlaced
        })
    }
}

// Implementation of missing module-level functions
/// Gets all display devices on macOS.
pub fn all_display_devices(_dispatcher: EventLoopDispatcher) -> Result<Vec<DisplayDevice>> {
    use objc2_core_graphics::{CGGetActiveDisplayList, CGGetOnlineDisplayList};
    
    const MAX_DISPLAYS: u32 = 32;
    let mut display_ids = vec![0u32; MAX_DISPLAYS as usize];
    let mut display_count = 0u32;

    unsafe {
        CGGetOnlineDisplayList(
            MAX_DISPLAYS,
            display_ids.as_mut_ptr(),
            &raw mut display_count,
        )
    }
    .map_err(|_| Error::DisplayEnumerationFailed("Failed to get online displays".to_string()))?;

    display_ids.truncate(display_count as usize);
    
    Ok(display_ids
        .into_iter()
        .map(DisplayDevice::new)
        .collect())
}

/// Gets active display devices on macOS.
pub fn active_display_devices(_dispatcher: EventLoopDispatcher) -> Result<Vec<DisplayDevice>> {
    use objc2_core_graphics::CGGetActiveDisplayList;
    
    const MAX_DISPLAYS: u32 = 32;
    let mut display_ids = vec![0u32; MAX_DISPLAYS as usize];
    let mut display_count = 0u32;

    unsafe {
        CGGetActiveDisplayList(
            MAX_DISPLAYS,
            display_ids.as_mut_ptr(),
            &raw mut display_count,
        )
    }
    .map_err(|_| Error::DisplayEnumerationFailed("Failed to get active displays".to_string()))?;

    display_ids.truncate(display_count as usize);
    
    Ok(display_ids
        .into_iter()
        .map(DisplayDevice::new)
        .collect())
}

/// Gets display from point.
pub fn display_from_point(point: Point, dispatcher: EventLoopDispatcher) -> Result<Display> {
    use objc2_core_graphics::{CGPoint, CGDisplayAtPoint};
    
    let cg_point = CGPoint {
        x: point.x as f64,
        y: point.y as f64,
    };
    
    let display_id = unsafe { CGDisplayAtPoint(cg_point) };
    if display_id == 0 {
        return Err(Error::DisplayNotFound);
    }

    // Find the corresponding NSScreen
    let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
    let screens = unsafe { NSScreen::screens(mtm) };
    
    for screen in screens {
        let screen_display_id = unsafe { NSScreen::display_id(screen) };
        if screen_display_id == display_id {
            let ns_screen = MainThreadRef::new(dispatcher, screen);
            return Ok(Display::new(ns_screen));
        }
    }

    Err(Error::DisplayNotFound)
}

/// Gets primary display on macOS.
pub fn primary_display(dispatcher: EventLoopDispatcher) -> Result<Display> {
    use objc2_core_graphics::CGMainDisplayID;
    
    let main_display_id = unsafe { CGMainDisplayID() };
    
    // Find the corresponding NSScreen
    let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
    let screens = unsafe { NSScreen::screens(mtm) };
    
    for screen in screens {
        let screen_display_id = unsafe { NSScreen::display_id(screen) };
        if screen_display_id == main_display_id {
            let ns_screen = MainThreadRef::new(dispatcher, screen);
            return Ok(Display::new(ns_screen));
        }
    }

    Err(Error::PrimaryDisplayNotFound)
}
```

#### 5. `/packages/wm-platform/src/platform_hook.rs`

**Changes Required:**
- Remove async from functions that don't need it
- Fix function signatures to match platform implementations
- Add proper error handling and logging

**Implementation Changes:**
```rust
/// Gets all displays on the system.
/// 
/// Returns a vector of all displays currently connected to the system,
/// including both active and inactive displays.
pub fn all_displays() -> crate::Result<Vec<Display>> {
    tracing::debug!("Enumerating all displays");
    
    #[cfg(target_os = "windows")]
    {
        platform_impl::all_displays().map(|displays| {
            displays.into_iter().map(Display::from_platform_impl).collect()
        })
    }
    
    #[cfg(target_os = "macos")]
    {
        let dispatcher = platform_impl::EventLoopDispatcher::current()?;
        platform_impl::all_displays(dispatcher).map(|displays| {
            displays.into_iter().map(Display::from_platform_impl).collect()
        })
    }
}

/// Gets all display devices on the system.
/// 
/// Returns a vector of all display devices, including physical and virtual devices.
/// This provides more detailed information than `all_displays()`.
pub fn all_display_devices() -> crate::Result<Vec<DisplayDevice>> {
    tracing::debug!("Enumerating all display devices");
    
    #[cfg(target_os = "windows")]
    {
        platform_impl::all_display_devices().map(|devices| {
            devices.into_iter().map(DisplayDevice::from_platform_impl).collect()
        })
    }
    
    #[cfg(target_os = "macos")]
    {
        let dispatcher = platform_impl::EventLoopDispatcher::current()?;
        platform_impl::all_display_devices(dispatcher).map(|devices| {
            devices.into_iter().map(DisplayDevice::from_platform_impl).collect()
        })
    }
}
```

#### 6. New File: `/packages/wm-platform/src/platform_impl/windows/display_ext.rs`

**Purpose:** Windows-specific display extensions and advanced functionality.

```rust
use windows::{
    Win32::{
        Graphics::Gdi::{GetDeviceCaps, HORZRES, VERTRES, LOGPIXELSX, LOGPIXELSY, BITSPIXEL},
        UI::WindowsAndMessaging::{GetSystemMetrics, SM_REMOTESESSION},
    },
};

use crate::{Display, DisplayDevice, Result, Error};

/// Windows-specific display extensions.
pub trait DisplayExtWindows {
    /// Gets the monitor handle (HMONITOR).
    fn hmonitor(&self) -> isize;
    
    /// Gets advanced display information.
    fn display_info(&self) -> Result<WindowsDisplayInfo>;
    
    /// Checks if this display is accessed via Remote Desktop.
    fn is_remote_session(&self) -> Result<bool>;
}

/// Windows-specific display device extensions.
pub trait DisplayDeviceExtWindows {
    /// Gets the Windows device path.
    fn device_path(&self) -> Result<String>;
    
    /// Gets the hardware registry key.
    fn registry_key(&self) -> Result<String>;
    
    /// Gets detailed device information.
    fn device_info(&self) -> Result<WindowsDeviceInfo>;
}

/// Windows-specific display information.
#[derive(Debug, Clone)]
pub struct WindowsDisplayInfo {
    pub device_name: String,
    pub device_context_dpi_x: u32,
    pub device_context_dpi_y: u32,
    pub horizontal_resolution: u32,
    pub vertical_resolution: u32,
    pub bits_per_pixel: u32,
    pub is_remote_session: bool,
}

/// Windows-specific device information.
#[derive(Debug, Clone)]
pub struct WindowsDeviceInfo {
    pub device_id: String,
    pub device_key: String,
    pub device_string: String,
    pub state_flags: u32,
    pub registry_path: String,
}

impl DisplayExtWindows for Display {
    fn hmonitor(&self) -> isize {
        self.inner.hmonitor()
    }
    
    fn display_info(&self) -> Result<WindowsDisplayInfo> {
        // Implementation would use GetDeviceCaps and other Windows APIs
        // to gather comprehensive display information
        todo!("Implement display_info")
    }
    
    fn is_remote_session(&self) -> Result<bool> {
        let is_remote = unsafe { GetSystemMetrics(SM_REMOTESESSION) };
        Ok(is_remote != 0)
    }
}

impl DisplayDeviceExtWindows for DisplayDevice {
    fn device_path(&self) -> Result<String> {
        // Implementation would query device interface
        Ok(self.inner.hardware_id.clone())
    }
    
    fn registry_key(&self) -> Result<String> {
        // Implementation would get registry key for device
        todo!("Implement registry_key")
    }
    
    fn device_info(&self) -> Result<WindowsDeviceInfo> {
        // Implementation would gather comprehensive device info
        todo!("Implement device_info")
    }
}
```

#### 7. New File: `/packages/wm-platform/src/platform_impl/macos/display_ext.rs`

**Purpose:** macOS-specific display extensions and advanced functionality.

```rust
use objc2_core_graphics::{CGDirectDisplayID, CGDisplayIOServicePort};
use objc2_app_kit::NSScreen;
use objc2::rc::Retained;

use crate::{Display, DisplayDevice, Result, Error, platform_impl::MainThreadRef};

/// macOS-specific display extensions.
pub trait DisplayExtMacos {
    /// Gets the Core Graphics display ID.
    fn cg_display_id(&self) -> CGDirectDisplayID;
    
    /// Gets the NSScreen instance.
    fn ns_screen(&self) -> Result<MainThreadRef<Retained<NSScreen>>>;
    
    /// Gets the IOService port for advanced operations.
    fn io_service_port(&self) -> Result<u32>;
    
    /// Checks if this is a built-in display.
    fn is_builtin(&self) -> Result<bool>;
    
    /// Gets display color profile information.
    fn color_profile(&self) -> Result<MacosColorProfile>;
}

/// macOS-specific display device extensions.
pub trait DisplayDeviceExtMacos {
    /// Gets the Core Graphics display ID.
    fn cg_display_id(&self) -> CGDirectDisplayID;
    
    /// Gets IOKit device information.
    fn iokit_info(&self) -> Result<MacosIOKitInfo>;
}

/// macOS color profile information.
#[derive(Debug, Clone)]
pub struct MacosColorProfile {
    pub name: String,
    pub description: String,
    pub white_point: (f64, f64),
    pub gamma: f64,
    pub color_space: String,
}

/// macOS IOKit device information.
#[derive(Debug, Clone)]
pub struct MacosIOKitInfo {
    pub service_port: u32,
    pub vendor_id: u32,
    pub product_id: u32,
    pub serial_number: String,
    pub manufacture_date: String,
}

impl DisplayExtMacos for Display {
    fn cg_display_id(&self) -> CGDirectDisplayID {
        self.inner.cg_display_id()
    }
    
    fn ns_screen(&self) -> Result<MainThreadRef<Retained<NSScreen>>> {
        Ok(self.inner.ns_screen().clone())
    }
    
    fn io_service_port(&self) -> Result<u32> {
        let service_port = unsafe { CGDisplayIOServicePort(self.inner.cg_display_id()) };
        Ok(service_port)
    }
    
    fn is_builtin(&self) -> Result<bool> {
        use objc2_core_graphics::CGDisplayIsBuiltin;
        let is_builtin = unsafe { CGDisplayIsBuiltin(self.inner.cg_display_id()) };
        Ok(is_builtin != 0)
    }
    
    fn color_profile(&self) -> Result<MacosColorProfile> {
        // Implementation would use ColorSync APIs
        todo!("Implement color_profile")
    }
}

impl DisplayDeviceExtMacos for DisplayDevice {
    fn cg_display_id(&self) -> CGDirectDisplayID {
        self.inner.cg_display_id()
    }
    
    fn iokit_info(&self) -> Result<MacosIOKitInfo> {
        // Implementation would use IOKit APIs
        todo!("Implement iokit_info")
    }
}
```

#### 8. New File: `/packages/wm-platform/tests/display_tests.rs`

**Purpose:** Comprehensive tests for the display API.

```rust
#[cfg(test)]
mod display_tests {
    use super::*;
    use crate::display::*;
    
    #[test]
    fn test_all_displays() {
        let displays = all_displays().expect("Failed to get displays");
        assert!(!displays.is_empty(), "Should have at least one display");
        
        // Test that each display has valid properties
        for display in &displays {
            let id = display.id();
            let name = display.name().expect("Display should have a name");
            let bounds = display.bounds().expect("Display should have bounds");
            let working_area = display.working_area().expect("Display should have working area");
            let scale_factor = display.scale_factor().expect("Display should have scale factor");
            let dpi = display.dpi().expect("Display should have DPI");
            let is_primary = display.is_primary().expect("Should be able to check if primary");
            
            assert!(!name.is_empty(), "Display name should not be empty");
            assert!(bounds.width > 0 && bounds.height > 0, "Display bounds should be positive");
            assert!(working_area.width > 0 && working_area.height > 0, "Working area should be positive");
            assert!(scale_factor > 0.0, "Scale factor should be positive");
            assert!(dpi > 0, "DPI should be positive");
            
            println!("Display: {} ({}x{}) DPI: {} Scale: {:.2} Primary: {}", 
                name, bounds.width, bounds.height, dpi, scale_factor, is_primary);
        }
    }
    
    #[test] 
    fn test_primary_display() {
        let primary = primary_display().expect("Should have a primary display");
        let is_primary = primary.is_primary().expect("Should be able to check if primary");
        assert!(is_primary, "Primary display should return true for is_primary()");
    }
    
    #[test]
    fn test_display_devices() {
        let devices = all_display_devices().expect("Failed to get display devices");
        assert!(!devices.is_empty(), "Should have at least one display device");
        
        for device in &devices {
            let id = device.id();
            let name = device.name().expect("Device should have a name");
            let connection_state = device.connection_state().expect("Device should have connection state");
            let refresh_rate = device.refresh_rate().expect("Device should have refresh rate");
            
            assert!(!name.is_empty(), "Device name should not be empty");
            assert!(refresh_rate > 0.0, "Refresh rate should be positive");
            
            println!("Device: {} State: {:?} Refresh: {:.1}Hz", 
                name, connection_state, refresh_rate);
        }
    }
    
    #[test]
    fn test_display_equality() {
        let displays = all_displays().expect("Failed to get displays");
        if displays.len() >= 2 {
            assert_ne!(displays[0], displays[1], "Different displays should not be equal");
        }
        
        // Test equality with same display
        if !displays.is_empty() {
            let display1 = &displays[0];
            let display2 = &displays[0];
            assert_eq!(display1, display2, "Same display should be equal to itself");
        }
    }
    
    #[test]
    fn test_display_from_point() {
        let displays = all_displays().expect("Failed to get displays");
        if let Some(display) = displays.first() {
            let bounds = display.bounds().expect("Display should have bounds");
            let center_point = Point {
                x: bounds.x + bounds.width / 2,
                y: bounds.y + bounds.height / 2,
            };
            
            let found_display = display_from_point(center_point)
                .expect("Should find display at center point");
            
            // The found display should be the same as the original
            assert_eq!(display.id(), found_display.id(), 
                "Display from point should match original display");
        }
    }
    
    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_extensions() {
        use crate::platform_impl::windows::DisplayExtWindows;
        
        let displays = all_displays().expect("Failed to get displays");
        if let Some(display) = displays.first() {
            let hmonitor = display.hmonitor();
            assert_ne!(hmonitor, 0, "HMONITOR should not be null");
        }
    }
    
    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_extensions() {
        use crate::platform_impl::macos::DisplayExtMacos;
        
        let displays = all_displays().expect("Failed to get displays");
        if let Some(display) = displays.first() {
            let cg_display_id = display.cg_display_id();
            assert_ne!(cg_display_id, 0, "CGDirectDisplayID should not be zero");
            
            let is_builtin = display.is_builtin().expect("Should be able to check if builtin");
            println!("Display is builtin: {}", is_builtin);
        }
    }
}
```

## Testing Strategy

### Unit Tests
- Test all public API functions
- Test error handling paths
- Test platform-specific extensions
- Test display enumeration consistency
- Test display property validation

### Integration Tests  
- Test display hot-plugging scenarios
- Test display mode changes
- Test multi-monitor setups
- Test primary display detection
- Test point-to-display mapping

### Performance Tests
- Benchmark display enumeration performance
- Test repeated calls for memory leaks
- Test concurrent access patterns

## Documentation Requirements

### API Documentation
- Comprehensive rustdoc for all public functions
- Platform-specific behavior notes
- Error conditions and handling
- Usage examples for common scenarios

### User Guide
- Getting started with display enumeration
- Working with multiple displays
- Platform-specific considerations
- Migration guide from old APIs

## Migration Strategy

### Phase 1: Core Implementation
1. Implement missing error types
2. Fix compilation errors in Windows implementation
3. Complete macOS implementations
4. Add basic unit tests

### Phase 2: Advanced Features
1. Add display mode management
2. Implement platform-specific extensions
3. Add comprehensive error handling
4. Expand test coverage

### Phase 3: Polish and Optimization
1. Performance optimization
2. Memory usage optimization
3. Comprehensive documentation
4. Integration testing

## Risk Mitigation

### Platform API Changes
- Use stable platform APIs where possible
- Add compatibility layers for API changes
- Comprehensive testing on different OS versions

### Memory Management
- Proper resource cleanup in all code paths
- RAII patterns for native resources
- Memory leak testing

### Thread Safety
- Clear documentation of thread safety guarantees
- Proper synchronization for shared resources
- Main thread requirements clearly marked

## Success Criteria

1. **Functionality**: All TODO items resolved, no compilation errors
2. **API Completeness**: Full feature parity across platforms
3. **Documentation**: 100% API documentation coverage
4. **Testing**: >90% test coverage, all tests passing
5. **Performance**: No significant performance regression
6. **Memory**: No memory leaks detected in testing
7. **Compatibility**: Works on Windows 10+ and macOS 11+

## Implementation Timeline

- **Week 1**: Core error handling and Windows implementation fixes
- **Week 2**: macOS implementation completion
- **Week 3**: API surface completion and basic testing
- **Week 4**: Advanced features and comprehensive testing
- **Week 5**: Documentation, polish, and final integration testing

This implementation plan provides a comprehensive roadmap for creating a production-ready display API that addresses all identified issues while following the project's coding standards and architectural patterns.

---

**Next Steps**: Review this plan and proceed with Phase 1 implementation, starting with error type definitions and Windows display API fixes.