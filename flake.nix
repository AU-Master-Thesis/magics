{
  description = "gbp-rs";

  inputs = {
    # wgsl_analyzer.url = "github:wgsl-analyzer/wgsl-analyzer";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import inputs.nixpkgs {inherit system overlays;};
      rust-extensions = [
        "rust-src"
        "rust-analyzer"
        "llvm-tools-preview" # used with `cargo-pgo`
      ];
      rust-additional-targets = ["wasm32-unknown-unknown"];

      python-deps = with pkgs.python3Packages; [
        numpy
        scipy
        rich
        tabulate
        matplotlib
        toolz
        seaborn
        result
        pretty-errors
        seaborn
        catppuccin
        jupyter
        notebook
        pip
      ];
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
        freetype
        fontconfig
        # wgsl-analyzer-pkgs.wgsl_analyzer
        # wgsl_analyzer.packages.${system}
        # wgsl_analyzer.outputs.packages.${system}.default
      ];
      cargo-subcommands = with pkgs; [
        cargo-bloat
        cargo-expand
        cargo-outdated
        cargo-show-asm
        cargo-make
        cargo-modules
        cargo-nextest
        cargo-rr
        cargo-udeps
        cargo-watch
        cargo-wizard
        cargo-pgo
        cargo-flamegraph
        cargo-license
        cargo-release
        # cargo-tree

        #   # cargo-profiler
        #   # cargo-feature
      ];
      rust-deps = with pkgs;
        [
          # rustup
          taplo # TOML formatter and LSP
          bacon
          mold # A Modern Linker
          clang # For linking
          gdb # debugger
          # lldb # debugger
          # rr # time-traveling debugger
          ra-multiplex
          trunk # rust wasm bundler
          wasm-bindgen-cli
          # binaryen # wasm-opt
          sass
          tailwindcss
          graphviz
          dot2tex
          blas
          # openblas
          openssl
          # lapack
          gcc
          gfortran
          zlib
        ]
        ++ cargo-subcommands;
      dev-deps = with pkgs; [
        nodejs
        nodejs
        just
        typos # spell checker
        act # run github actions local in a docker container
        dot-language-server
        gh
        texliveFull
      ];
    in
      with pkgs; {
        formatter.${system} = pkgs.alejandra;
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = [pkgs.pkg-config];
          buildInputs =
            [
              # (rust-bin.stable.latest.default.override
              #   {
              #     extensions = rust-extensions;
              #     targets = rust-targets;
              #   })
              # (rust-bin.beta.latest.default.override {
              #     extensions = ["rust-src" "rust-analyzer"];
              # })
              (
                rust-bin.selectLatestNightlyWith (toolchain:
                  toolchain.default.override {
                    extensions =
                      rust-extensions
                      ++ [
                        "rustc-codegen-cranelift-preview"
                      ];
                    targets = ["wasm32-unknown-unknown"];
                  })
              )
            ]
            ++ bevy-deps
            ++ rust-deps
            ++ dev-deps
            ++ python-deps;

          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };
      });
}
