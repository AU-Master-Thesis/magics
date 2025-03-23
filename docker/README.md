# Docker Environment for Magics Project

This directory contains Docker configuration for the Magics project with GPU support.

## Files

- `Dockerfile`: Defines the container image with Rust 1.87, Python 3.10, and GPU support
- `docker-compose.yml`: Configures the service with volume mounting and GPU access
- `entrypoint.sh`: Sets up the environment when the container starts
- `.dockerignore`: Excludes unnecessary files from the build context

## Requirements

- Docker
- Docker Compose
- NVIDIA Container Toolkit (for GPU support)

## Usage

### Building the Docker Image

From the `docker` directory:

```bash
docker-compose build
```

### Running the Container

From the `docker` directory:

```bash
docker-compose run --rm magics
```

### For GUI Applications

Enable X11 forwarding:

```bash
xhost +local:docker
docker-compose run --rm magics
```

### Building Your Project

Inside the container:

```bash
cd /home/developer/magics
cargo build
```

### Running Your Application

Inside the container:

```bash
cargo run
```

## Notes

- The Docker environment is configured to use your GPU for Bevy rendering through Vulkan.
- All project files are mounted from your host system, so you can edit them with your preferred editor outside the container while building and running inside the container.
- The Cargo registry is cached in a Docker volume for faster builds.
