# Technical Context

## Technology Stack
1. Programming Language
   - Rust (1.78+)
   - Cargo build system
   - Rust Workspace with multiple crates
   - Python (3.8+) for API integration

2. Game Engine and Rendering
   - Bevy (v0.13)
   - Egui for user interface
   - Vulkan rendering backend

3. External Libraries
   - ndarray for linear algebra
   - parry for collision detection
   - serde for serialization
   - clap for CLI parsing
   - rand for random number generation
   - zmq for network communication
   - serde_json for message serialization

4. Python Integration
   - OpenAI Gym for reinforcement learning interface
   - NumPy for numerical operations
   - pyzmq for ZeroMQ communication
   - Gymnasium compatibility

## Development Environment
- Rust toolchain
- VSCode with Rust-Analyzer
- Nix/NixOS for dependency management
- direnv for environment configuration
- GitHub for version control
- Python development tools and virtual environments

## Build and Compilation
- Cargo workspaces
- Multiple crate architecture
- Release and debug configurations
- Fat LTO (Link Time Optimization)
- Incremental compilation support
- Feature flags for API functionality

## API Architecture
- ZeroMQ for client-server communication
- REQ-REP socket pattern
- JSON message serialization
- Thread-safe state access via ApiState
- Step-based simulation control
- Factor graph weight modification

## Performance Optimization
- Mimalloc memory allocator
- Zero-cost abstractions
- SIMD optimizations
- Compile-time generics
- Minimal runtime overhead
- Efficient state extraction
- Optimized message serialization

## Dependency Management
- Workspace-level dependency configuration
- Explicit feature flags
- Minimal external dependencies
- Carefully selected performance-oriented libraries
- Python dependency management via pip/setuptools

## Cross-Platform Considerations
- Linux primary development platform
- WSL2 support
- X11 and Wayland compatibility
- Python compatibility across platforms

## Testing and Validation
- Unit testing for individual components
- Integration testing for factor graph
- Performance benchmarking
- Continuous Integration (GitHub Actions)
- Python API test suite

## Configuration and Flexibility
- TOML and RON configuration formats
- Command-line interface with rich options
- Environment-based configuration
- Scenario and experiment customization
- Python-based experiment configuration

## Security and Safety
- Memory safety through Rust's ownership model
- No unsafe code where possible
- Explicit error handling
- Comprehensive logging
- Default to localhost-only binding for ZMQ server
