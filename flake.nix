{
  description = "gbp-rs";
  inputs = {
    # wgsl_analyzer.url = "github:wgsl-analyzer/wgsl-analyzer";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    # wgsl_analyzer,
    ...
  } @ inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import inputs.nixpkgs {inherit system;};
      # wgsl-analyzer-pkgs = import inputs.wgsl_analyzer {inherit system;};
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
        egl-wayland
        # wgsl-analyzer-pkgs.wgsl_analyzer
        # wgsl_analyzer.packages.${system}
        # wgsl_analyzer.outputs.packages.${system}.default
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
        cargo-rr
        cargo-udeps

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
          gdb # debugger
          # lldb # debugger
          rr # time-traveling debugger
        ]
        ++ cargo-subcommands;
    in
      with pkgs; {
        formatter.${system} = pkgs.alejandra;
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            pkgs.pkg-config
            # cargo
            # rustc
          ];
          buildInputs =
            [
              nodejs
              just
              d2
              graphviz
              dot-language-server
              openblas
              openssl
              # lapack
              gcc
              gfortran
            ]
            ++ bevy-deps
            ++ rust-deps;

          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };
      });
}
