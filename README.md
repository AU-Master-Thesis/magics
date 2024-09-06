# magics 

> Master Thesis Project in Computer Engineering at Aarhus University 2024 on "Simulating Multi-agent Path Planning in Complex environments using Gaussian Belief Propagation and Global Path Finding". Available here (https://drive.google.com/file/d/12g-7bqcy_yfkZdpKzxQAErayFJQhu4sE/view?usp=sharing)

## Demo

> The video below demonstrates some of the features of the simulation tool, and shows how the GBP algorithm can handle complex scenarios such as a multi-lane twoway junction.

[magics-functionality-demo-trimmed-for-github.webm](https://github.com/user-attachments/assets/8f5d0db6-dd2c-41a3-9a12-4ccddf80d4f3)

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

## Credits

The primary algorithm for GBP path planning is based on [gbpplanner](https://github.com/aalpatya/gbpplanner) by [Aalok Patwardhan](https://aalok.uk/) from  Imperial College London and Dyson Robotics Lab. As part of this thesis we have reimplemented and extended upon in Rust!

