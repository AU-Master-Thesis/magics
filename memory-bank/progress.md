# Project Progress

## Completed Features
- [x] Factor Graph Architecture
- [x] Message Passing Mechanism
- [x] Basic Gaussian Belief Propagation
- [x] Multi-agent Simulation Framework
- [x] Collision Avoidance Prototype
- [x] CLI and Configuration Support
- [x] Basic Visualization Tools
- [x] Pause/Play Functionality
- [x] ZeroMQ API Core Implementation
  - [x] ZeroMQ dependencies integration
  - [x] Message protocol design and implementation
  - [x] ZMQ server implementation
  - [x] API plugin conditional loading
  - [x] Simulation stepping mechanism
  - [x] Virtual time advancement using FixedUpdate counting
  - [x] Config Hz access via API
  - [x] Basic agent and environment data extraction
  - [x] Collision detection and tracking
  - [x] Agent State Extraction
    - [x] Basic position and velocity
    - [x] Factor graph state
    - [x] Mission state and progress
    - [x] Neighbor connections
    - [x] Communication properties
    - [x] Collision information
- [x] Factor Graph Details Extraction
  - [x] Variable information
  - [x] Factor information (obstacle, interrobot, tracking, dynamic)
  - [x] Message statistics
  - [x] Agent Spawning (`SpawnAgent` command)
  - [x] Agent Removal (`RemoveAgent` command)
  - [x] Current Scenario Query (`GetCurrentScenario` command)

## Partially Implemented Features
- [x] Python Client Implementation
  - [x] Basic client structure
  - [x] Added `spawn_agent`, `remove_agent`, and `get_current_scenario` methods
  - [x] Connection handling
  - [x] Command sending/receiving
  - [x] Basic step and reset methods
  - [ ] Full OpenAI Gym integration


## Important Next Steps

1. **Finalize OpenAI Gym Integration**
   - Define observation and action spaces
   - Implement proper reward calculation
   - Create render methods
   - Ensure proper state conversion between Rust and Python

## Pending Features
- [ ] Advanced Reinforcement Learning Support
- [ ] Real-time Performance Visualization for ML (not important now)
- [ ] Expanded Reward Functions
- [ ] Distributed Multi-robot Coordination

## Known Issues
- Performance bottlenecks in message passing (Huge problem in the iterate_gbp_v2 function, too many allocations, copying ndarray matricer, and deallocation aka. drop)
- Limited scenario complexity
- Potential numerical stability challenges

## Performance Metrics
- Current Scenario Complexity: Medium
- Average Robot Count: 30-100
- Computation Time per Iteration: ~10-20ms
- Collision Avoidance Success Rate: ~95%

## Research Milestones
1. **Algorithm Design and Initial Implementation** ✅
   - [x] Factor Graph Architecture
   - [x] Basic Message Passing
   - [x] Initial Simulation Framework

2. **Performance and Optimization** ⏳
   - [ ] Computational Complexity Analysis
   - [ ] Parallel Processing Integration
   - [ ] Memory Efficiency Improvements

3. **API and Reinforcement Learning Integration** ⏳
   - [x] ZeroMQ API Implementation
   - [x] Basic Python Client Implementation
   - [ ] OpenAI Gym Environment Creation
   - [ ] Reward Function Development
   - [ ] Integration with RL Algorithms

4. **Validation and Experimentation** 🔜
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
