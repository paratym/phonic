{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
      in {
        devShells.default = pkgs.mkShell (with pkgs; {
          packages = [
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rustfmt" ];
            })
            rust-analyzer
            pkg-config
            ffmpeg_4-full
            clang
            libclang
            alsa-lib
          ];
          shellHook = ''
            export LIBCLANG_PATH="${libclang.lib}/lib"
          '';
        });
      });
}
