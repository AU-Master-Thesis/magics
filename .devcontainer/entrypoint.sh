#!/bin/bash
set -e

# Setup for NVIDIA GPU (Vulkan)
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/nvidia_icd.json

# Fix permissions on Cargo cache volume
chown -R developer:developer /home/developer/.cargo

# Ensure cargo environment variables are set (standard rustup setup)
export PATH="/home/developer/.cargo/bin:${PATH}"

# Switch to non-root user and run provided commands
exec gosu developer "$@"
docker builder prune -af
docker build --no-cache -f .devcontainer/Dockerfile -t test-entrypoint .
docker run --rm test-entrypoint ls -la /entrypoint.sh
