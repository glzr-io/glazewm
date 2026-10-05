fn main() {
  println!("cargo:rerun-if-env-changed=VERSION_NUMBER");

  // Re-exported through the build script rather than read straight out of
  // the environment with `env!`, since cargo doesn't track the latter for
  // rebuilds. Without this, changing the version number leaves the
  // previous one baked into an already-compiled crate.
  let version = std::env::var("VERSION_NUMBER")
    .unwrap_or_else(|_| "0.0.0".to_string());

  println!("cargo:rustc-env=VERSION_NUMBER={version}");
}
