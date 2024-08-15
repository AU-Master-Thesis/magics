# gbp-rs
https://github.com/aalpatya/gbpplanner reimplemented and extended in Rust!

## Thesis

The accompanying thesis is available online [here](https://drive.google.com/file/d/12g-7bqcy_yfkZdpKzxQAErayFJQhu4sE/view?usp=sharing).

## External Dependencies

Most dependencies used are available through the `crates.io` registry. And should work on all major platforms supported by the `cargo` build tool. Still some external dependencies are needed for the graphical session.

| Dependencies | Platform Specific |
|--------------|----------|
| `udev` | Linux |
| `alsa-lib` | Linux |
| `vulkan-loader` |  |
| `xorg.libX11` | Linux + X11 |
| `xorg.libXcursor` | Linux + X11 |
| `xorg.libXi` | Linux + X11 |
| `xorg.libXrandr` | Linux + X11 |
| `libxkbcommon` | Linux + X11 |
| `wayland` | Linux + Wayland |
| `egl-wayland` | Linux + Wayland |
| `freetype` | |
| `fontconfig` |  |

The exact name of the dependency might vary between platforms, and even between Linux distributions. Consult the respective package management tool used on your system for their exact names.


### Nix/NixOS

The `./flake.nix` file provides a development shell with all the necessary dependencies to run the project. If you have `direnv` installed you can simply use the provided `.envrc` and type `direnv allow` to automatically enter it. Otherwise you can run:

```sh
# To enter the development environment
nix develop
```

## Build

The entire project can be build with the following command:

```
cargo build --release
```

## Run


```sh
cargo run --release --bin magics # Open the simulator
cargo run --release --bin magics -- --list # List all available scenarios

# Run a specific scenario
cargo run --release --bin magics -- -i <SCENARIO_NAME>
cargo run --release --bin magics -- --initial-scenario <SCENARIO_NAME>
```
