fn main() {
  let mut res = tauri_winres::WindowsResource::new();
  res.set_icon("../../resources/icon.ico");
  res.compile().unwrap();
}
