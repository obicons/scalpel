{ pkgs ? import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/d540c6348227dff41a708db5c0b70cc3018080ea.tar.gz")) {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "scalpel";
  version = "1.0.0";

  src = builtins.path { path = ./.; name = "scalpel"; };

  cargoHash = "sha256-rkxM4fay8jr52Xal+umGjNXaBM9lt6R5C2M6nzA8u6A=";

  buildInputs = [ pkgs.z3_4_11 ];
}
