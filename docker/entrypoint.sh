#!/bin/bash
set -e

# Setup for NVIDIA GPU
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/nvidia_icd.json

# Execute the command passed to docker run
exec "$@"
