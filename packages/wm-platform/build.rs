fn main() {
  #[cfg(target_os = "macos")]
  println!(
    "cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks"
  );
}
