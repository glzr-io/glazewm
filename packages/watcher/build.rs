fn main() {
  let mut res = tauri_winres::WindowsResource::new();

  res.set_icon("../../resources/icon.ico");

  // Set language to English (US).
  res.set_language(0x0409);

  res.set("OriginalFilename", "glazewm-watcher.exe");
  res.set("ProductName", "GlazeWM Watcher");
  res.set("FileDescription", "GlazeWM Watcher");
  res.set("FileVersion", "0.0.0");
  res.set("ProductVersion", "0.0.0");

  res.compile().unwrap();
}
