# Dev Container for Magics Project

This directory contains the Dev Container configuration for the Magics project with GPU support.

## Files

- `Dockerfile`: Defines the container image with Rust nightly, Python 3.10, and GPU support
- `docker-compose.yml`: Configures the service with volume mounting and GPU access
- `devcontainer.json`: VSCode Dev Container configuration
- `entrypoint.sh`: Sets up the environment when the container starts
- `.dockerignore`: Excludes unnecessary files from the build context

## Requirements

- Docker
- Docker Compose
- NVIDIA Container Toolkit (for GPU support)
- VSCode with Dev Containers extension

## Usage

### Using with VSCode

1. Open the project in VSCode
2. Click on the "Reopen in Container" button when prompted
3. VSCode will build the container and open the project inside it

### Manual Building and Running

```bash
# From the project root directory
docker-compose -f .devcontainer/docker-compose.yml build
```

### Running the Container Manually

```bash
# From the project root directory
docker-compose -f .devcontainer/docker-compose.yml run --rm magics
```

### For GUI Applications

Enable X11 forwarding:

```bash
xhost +local:docker
docker-compose -f .devcontainer/docker-compose.yml run --rm magics
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
