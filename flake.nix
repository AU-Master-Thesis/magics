{
  description = "gbp-rs";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  } @ inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import inputs.nixpkgs {inherit system;};
      bevy-deps = with pkgs; [
        udev
        alsa-lib
        vulkan-loader
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr
        libxkbcommon
        wayland
      ];
      cargo-subcommands = with pkgs; [
        cargo-bloat
        cargo-expand
        cargo-info
        cargo-outdated
        cargo-show-asm
        cargo-nextest
        cargo-modules
        cargo-watch

        #   # cargo-profiler
        #   # cargo-feature
      ];
      rust-deps = with pkgs;
        [
          rustup
          taplo # TOML formatter and LSP
          bacon
          mold # A Modern Linker
          clang # For linking
        ]
        ++ cargo-subcommands;
    in
      with pkgs; {
        formatter.${system} = pkgs.alejandra;
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = [
            pkgs.pkg-config
          ];
          buildInputs = [just d2] ++ bevy-deps ++ rust-deps;

          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };
      });
}
