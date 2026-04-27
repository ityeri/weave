{
  description = "x11 native libs environments for macroquad crate";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, utils, rust-overlay }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        runtimeLibs = with pkgs; [
          libGL
          libX11
          libXcursor
          libXi
          libXrandr
          libxkbcommon
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
            })
            runtimeLibs
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath runtimeLibs}"
            export RUST_SRC_PATH="${pkgs.rustPlatform.rustLibSrc}"
          '';
        };
      });
}
