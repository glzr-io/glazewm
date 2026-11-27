fn main() {
  // Build scripts run on the host, so `cfg` would check the host OS.
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    println!(
      "cargo:rustc-link-search=framework=/System/Library/PrivateFrameworks"
    );
  }
}
