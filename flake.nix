{
  description = "gbp-rs";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixpkgs.inputs.flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      # system = "x86_64-linux";
      # pkgs = nixpkgs.legacyPackages.${system};
      pkgs = import nixpkgs {inherit system;};
    in
      with pkgs; {
        # devShells.${system}.default = pkgs.mkShell rec {
        # formatter.${system} = pkgs.alejandra;
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = [
            pkgs.pkg-config
          ];
          buildInputs = [
            pkgs.udev
            pkgs.alsa-lib
            pkgs.vulkan-loader
            pkgs.xorg.libX11
            pkgs.xorg.libXcursor
            pkgs.xorg.libXi
            pkgs.xorg.libXrandr # To use the x11 feature
            pkgs.libxkbcommon
            pkgs.wayland # To use the wayland feature

            pkgs.bacon
            pkgs.rustup
          ];

          # shellHook = ''
          #   echo hello
          # '';

          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };
      });
}
