let
  # Pinned nixpkgs, deterministic. Last updated: 08/04/2023.
  pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/d540c6348227dff41a708db5c0b70cc3018080ea.tar.gz")) {};
in pkgs.mkShell {
  buildInputs = [ pkgs.cargo pkgs.iconv pkgs.rustc pkgs.rust-analyzer pkgs.z3_4_11 ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
