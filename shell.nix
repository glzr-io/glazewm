let
  rust-overlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  pkgs = import <nixpkgs> {
    overlays = [(import rust-overlay)];
  };
  toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in
  pkgs.mkShell rec {
    packages = [
      toolchain
      pkgs.rust-analyzer-unwrapped
    ];

    RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";

    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = with pkgs; [
      # Rust
      clang
      # Replace llvmPackages with llvmPackages_X, where X is the latest LLVM version (at the time of writing, 16)
      llvmPackages.bintools
      rustup

      # Crate deps
      systemd
      seatd
      gtk3
    ];
  }
