FROM nvidia/cuda:12.3.1-base-ubuntu22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/home/developer/.rustup
ENV CARGO_HOME=/home/developer/.cargo
ENV PATH=$CARGO_HOME/bin:$PATH

# Install system dependencies (including gosu for safe user-switching)
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    git \
    gosu \
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
    libasound2-dev \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash developer

# Install Rust and Python as non-root user
USER developer
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly \
    && rustup component add rustfmt clippy \
    && rustup target add x86_64-unknown-linux-gnu

RUN python3.10 -m pip install --user --upgrade pip setuptools wheel numpy pyzmq gymnasium

# Set workspace directory
RUN mkdir -p /home/developer/magics
WORKDIR /home/developer/magics

# Copy entrypoint.sh correctly
USER root
COPY .devcontainer/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["sleep", "infinity"]
