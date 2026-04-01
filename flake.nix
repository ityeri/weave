{
  description = "Rust SDL2 multi-platform flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            pkg-config
            SDL2
            SDL2_image
            SDL2_mixer
            SDL2_ttf
          ];

          shellHook = ''
            export PKG_CONFIG_PATH="${pkgs.SDL2.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"

            ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
              export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (with pkgs; [ SDL2 SDL2_image SDL2_mixer SDL2_ttf ])}:$LD_LIBRARY_PATH"
            ''}

            ${pkgs.lib.optionalString pkgs.stdenv.isDarwin ''
              export DYLD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (with pkgs; [ SDL2 SDL2_image SDL2_mixer SDL2_ttf ])}:$DYLD_LIBRARY_PATH"
            ''}
          '';
        };
      });
}
