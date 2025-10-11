let
  nixpkgsVer = "bafe987a29b8bea2edbb3aba76b51464b3d222f0";
  pkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/${nixpkgsVer}.tar.gz") { config = {}; overlays = []; };
  libs = with pkgs; [
    openssl
  ];
in pkgs.mkShell {
  name = "processr";

  buildInputs = libs ++ (with pkgs; [
    cargo
    cargo-expand
    cargo-llvm-cov
    cargo-binutils
    rustc.llvmPackages.llvm
    rustc
    gcc
    rustfmt
    pkgconf
  ]);

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  RUST_BACKTRACE = 1;
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libs;
}
