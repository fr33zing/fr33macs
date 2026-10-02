{
  lib,
  fetchFromGitHub,
  rustPlatform,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "sexp-depth-vis";
  version = "0.1.0";
  src = ./.;
  cargoHash = "";
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
})
