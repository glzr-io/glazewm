use tauri_winres::VersionInfo;

fn main() {
  if cfg!(not(target_os = "windows")) {
    panic!("wm-watcher is only supported on Windows.");
  }

  println!("cargo:rerun-if-env-changed=VERSION_NUMBER");
  let mut res = tauri_winres::WindowsResource::new();

  res.set_icon("../../resources/assets/icon.ico");

  // Set language to English (US).
  res.set_language(0x0409);

  res.set("OriginalFilename", "glazewm-watcher.exe");
  res.set("ProductName", "GlazeWM Watcher");
  res.set("FileDescription", "GlazeWM Watcher");

  let version_parts = version_number()
    .split('.')
    .take(3)
    .map(|part| part.parse().unwrap_or(0))
    .collect::<Vec<u16>>();

  let [major, minor, patch] =
    <[u16; 3]>::try_from(version_parts).unwrap_or([0, 0, 0]);

  let version_str = format!("{major}.{minor}.{patch}.0");
  res.set("FileVersion", &version_str);
  res.set("ProductVersion", &version_str);

  let version_u64 = (u64::from(major) << 48)
    | (u64::from(minor) << 32)
    | (u64::from(patch) << 16);

  res.set_version_info(VersionInfo::FILEVERSION, version_u64);
  res.set_version_info(VersionInfo::PRODUCTVERSION, version_u64);

  res.compile().unwrap();
}

/// Reads the version number to stamp into the binary.
///
/// Read from the environment at build-script runtime rather than with
/// `env!`, since cargo doesn't track `env!` for rebuilds. The value is
/// re-exported with `cargo:rustc-env` so that the crate itself is also
/// rebuilt when the version changes.
fn version_number() -> String {
  let version = std::env::var("VERSION_NUMBER")
    .unwrap_or_else(|_| "0.0.0".to_string());

  println!("cargo:rustc-env=VERSION_NUMBER={version}");
  version
}
