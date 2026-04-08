use windows::{
  core::w,
  Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY,
    HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY, REG_EXPAND_SZ, REG_SZ,
    REG_VALUE_TYPE,
  },
};

const WINDOWS_11_BUILD_NUMBER: u32 = 22_000;

/// Gets whether the current Windows version is Windows 11 or newer.
pub(crate) fn is_windows_11_or_greater() -> crate::Result<bool> {
  let build_number = current_build_number()?;
  Ok(is_windows_11_or_greater_build(build_number))
}

/// Gets whether the given Windows build number is Windows 11 or newer.
#[must_use]
fn is_windows_11_or_greater_build(build_number: u32) -> bool {
  build_number >= WINDOWS_11_BUILD_NUMBER
}

/// Gets the current Windows build number from the registry.
fn current_build_number() -> crate::Result<u32> {
  let mut key = HKEY::default();

  unsafe {
    RegOpenKeyExW(
      HKEY_LOCAL_MACHINE,
      w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion"),
      0,
      KEY_READ | KEY_WOW64_64KEY,
      &mut key,
    )?;
  }

  let result = (|| {
    let mut value_type = REG_VALUE_TYPE(0);
    let mut buffer = [0u16; 32];
    #[allow(clippy::cast_possible_truncation)]
    let mut buffer_len =
      (buffer.len() * std::mem::size_of::<u16>()) as u32;

    unsafe {
      RegQueryValueExW(
        key,
        w!("CurrentBuildNumber"),
        None,
        Some(&mut value_type),
        Some(buffer.as_mut_ptr().cast()),
        Some(&mut buffer_len),
      )?;
    }

    if value_type != REG_SZ && value_type != REG_EXPAND_SZ {
      return Err(crate::Error::Platform(
        "Unexpected registry type for CurrentBuildNumber.".to_string(),
      ));
    }

    let char_len =
      usize::try_from(buffer_len)? / std::mem::size_of::<u16>();
    let build_str =
      String::from_utf16_lossy(&buffer[..char_len.saturating_sub(1)]);

    build_str.parse::<u32>().map_err(|err| {
      crate::Error::Platform(format!(
        "Failed to parse Windows build number '{build_str}': {err}.",
      ))
    })
  })();

  unsafe {
    RegCloseKey(key)?;
  }

  result
}

#[cfg(test)]
mod tests {
  use super::is_windows_11_or_greater_build;

  #[test]
  fn detects_windows_10_builds() {
    assert!(!is_windows_11_or_greater_build(19_045));
  }

  #[test]
  fn detects_windows_11_builds() {
    assert!(is_windows_11_or_greater_build(22_000));
    assert!(is_windows_11_or_greater_build(26_100));
  }
}
