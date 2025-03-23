# Base image with CUDA support
FROM nvidia/cuda:12.3.1-base-ubuntu22.04

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH
ENV RUST_VERSION=1.87.0

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    git \
    libssl-dev \
    pkg-config \
    python3.10 \
    python3.10-dev \
    python3.10-venv \
    python3-pip \
    libzmq3-dev \
    libvulkan-dev \
    vulkan-tools \
    mesa-vulkan-drivers \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libfontconfig-dev \
    libwayland-dev \
    libx11-xcb-dev \
    libxcb-dri2-0-dev \
    libxcb-dri3-dev \
    libxcb-present-dev \
    libxcb-sync-dev \
    libxshmfence-dev \
    libxrandr-dev \
    libglew-dev \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain $RUST_VERSION \
    && rustup component add rustfmt clippy

# Set up Python
RUN python3.10 -m pip install --upgrade pip \
    && python3.10 -m pip install setuptools wheel numpy pyzmq gymnasium

# Create a non-root user
RUN useradd -m -s /bin/bash developer
USER developer
WORKDIR /home/developer

# Set up project directory
RUN mkdir -p /home/developer/magics
WORKDIR /home/developer/magics

# Set up Rust environment for the project
RUN rustup default $RUST_VERSION \
    && rustup target add x86_64-unknown-linux-gnu

# Set up entrypoint
USER root
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
USER developer

ENTRYPOINT ["/entrypoint.sh"]
CMD ["bash"]
