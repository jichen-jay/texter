{ pkgs ? import <nixpkgs> {} }:

pkgs.pkgsStatic.rustPlatform.buildRustPackage rec {
  pname = "texter";
  version = "0.1.1";

  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
  
  cargoBuildTarget = "x86_64-unknown-linux-musl";

  nativeBuildInputs = [
    pkgs.pkgsStatic.pkg-config
    pkgs.pkgsStatic.openssl
  ];

  NIX_CFLAGS_LINK = "-static";
  
  # Optimization flags
  RUSTFLAGS = "-C opt-level=z -C target-cpu=native -C codegen-units=1";
  CARGO_PROFILE_RELEASE_LTO = "thin";
  
  # Strip binary
  stripAllList = [ "bin" ];
  
  # Enable automatic store optimization
  enableParallelBuilding = true;
}
