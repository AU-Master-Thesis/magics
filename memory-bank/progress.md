# Project Progress

## Implemented Features
- [x] Factor Graph Architecture
- [x] Message Passing Mechanism
- [x] Basic Gaussian Belief Propagation
- [x] Multi-agent Simulation Framework
- [x] Collision Avoidance Prototype
- [x] CLI and Configuration Support
- [x] Basic Visualization Tools
- [x] Pause/Play Functionality

## In Progress
- [ ] ZeroMQ API Integration
  - [x] Add ZeroMQ dependencies
  - [x] Create message protocol
  - [x] Implement ZMQ server
  - [x] Fix API plugin conditional loading
  - [ ] Fix environment loading issues
- [ ] OpenAI Gym Environment
  - [x] Create Python client
  - [ ] Implement observation and action spaces
  - [ ] Create step, reset, render methods
  - [ ] Add reward calculation
- [ ] Scenario Organization
  - [ ] Rename scenario folders to PascalCase (no spaces)
  - [ ] Update all references in config files

## Pending Features
- [ ] Per-agent Factor Graph Weights
- [ ] Advanced Reinforcement Learning Support
- [ ] Real-time Performance Visualization for ML
- [ ] Expanded Reward Functions
- [ ] Distributed Multi-robot Coordination
- [ ] Investigate how robots are moving. We might want to control x amount of fixed update per API step call.

## Current Tasks

### 1. API Feature Fix
- [x] Modify main.rs to conditionally add ApiPlugin only when the "api" feature is enabled
- [ ] Test running without the API feature to ensure it doesn't auto-pause

### 2. Environment Loading Investigation
- [ ] Check environment file contents
- [ ] Verify rendering systems are working correctly
- [ ] Test with a simple environment to isolate the issue

### 3. Scenario Folder Renaming
- [ ] Create a systematic approach to rename all folders
- [ ] Update all config files to reference the new paths
- [ ] Test to ensure all scenarios load correctly

## Known Issues
- Performance bottlenecks in message passing
- Limited scenario complexity
- Potential numerical stability challenges
- Thread safety concerns with concurrent API access
- Environment not loading properly in scenarios

## Performance Metrics
- Current Scenario Complexity: Medium
- Average Robot Count: 5-10
- Computation Time per Iteration: ~10-20ms
- Collision Avoidance Success Rate: ~95%

## Research Milestones
1. Algorithm Design and Initial Implementation
   - [x] Factor Graph Architecture
   - [x] Basic Message Passing
   - [x] Initial Simulation Framework

2. Performance and Optimization
   - [ ] Computational Complexity Analysis
   - [ ] Parallel Processing Integration
   - [ ] Memory Efficiency Improvements

3. API and Reinforcement Learning Integration
   - [x] ZeroMQ API Implementation
   - [ ] OpenAI Gym Environment Creation
   - [ ] Reward Function Development
   - [ ] Integration with RL Algorithms

4. Validation and Experimentation
   - [ ] Comprehensive Test Suite
   - [ ] Benchmark Scenario Development
   - [ ] Performance Comparison with Existing Approaches
   - [ ] Reinforcement Learning Experiments

## Future Exploration
- Adaptive Belief Propagation
- Advanced Machine Learning Integration
- Distributed Multi-robot Reinforcement Learning
- Real-world Robot Interface
- Online Learning for Weight Optimization
