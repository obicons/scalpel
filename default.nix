{ pkgs ? import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/d540c6348227dff41a708db5c0b70cc3018080ea.tar.gz")) {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "scalpel";
  version = "1.0.0";

  src = builtins.path { path = ./.; name = "scalpel"; };

  cargoHash = "sha256-46+p3hgmD0OTj0fY5Q8exW0cHIAEtN0ByBddiTvpElU=";

  buildInputs = [ pkgs.z3_4_11 ];
}
