let
  nixpkgsVer = "0ff09db9d034a04acd4e8908820ba0b410d7a33a";
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
