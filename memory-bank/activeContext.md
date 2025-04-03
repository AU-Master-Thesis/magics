# Active Context

## Current Development Focus
The current development focus is on finalizing the API implementation and creating comprehensive documentation to support reinforcement learning research:

1. **API Completion**: Finalizing the remaining aspects of the API implementation, reset environment, load environment.

1.5. **Simulation extensions**:  Also working on a new config variable that pauses the simulation on load. (We have one that pauses on spawn, but this is not the logic we want.)  

2. **Documentation Structure**: Creating a structured documentation system with multiple interconnected files to make the API more accessible and easier to understand, starting with `crates/magics/src/api/API_DOCUMENTATION.md`.

3. **OpenAI Gym Integration**: Completing the integration with OpenAI Gym to enable reinforcement learning research, building on the existing Python client (`python_api/magics_client.py`).

4. **Memory Bank Updates**: Ensuring all project documentation accurately reflects the current state of implementation.

## Implementation Status

### Completed Components
- Core ZeroMQ API implementation with server (`crates/magics/src/api/zmq_server.rs`) and message protocol (`crates/magics/src/api/message.rs`)
- Basic Python client (`python_api/magics_client.py`) with connection handling and command sending/receiving
- Agent state extraction (`crates/magics/src/api/extract.rs`) including position, velocity, factor graph state, and mission information
- Factor graph details extraction (`crates/magics/src/api/factor_details.rs`) including variables and factors
- Collision detection and tracking (via `PreviousCollisionCounts` in `crates/magics/src/api/plugin.rs`)
- Simulation stepping mechanism with configurable iterations (`crates/magics/src/api/plugin.rs`)
- Config Hz access via API (`crates/magics/src/api/state.rs` and `crates/magics/src/api/zmq_server.rs`)
- Per-agent factor graph weights (`crates/magics/src/factorgraph/factorgraph.rs` and `crates/magics/src/api/weights.rs`)
- Agent spawning via API (`SpawnAgent` command, `handle_agent_spawn_requests` system in `plugin.rs`)
- Agent removal via API (`RemoveAgent` command, `handle_agent_removal_requests` system in `plugin.rs`)
- Current scenario query via API (`GetCurrentScenario` command, `zmq_server.rs`, `state.rs`, `plugin.rs`)

### Partially Implemented Components
- Python client (`python_api/magics_client.py` - Added `spawn_agent` and `remove_agent` methods, but still missing full OpenAI Gym integration)

## Key Files and Their Purposes
- `crates/magics/src/api/mod.rs`: Main module definition and exports
- `crates/magics/src/api/plugin.rs`: Bevy plugin for API integration
- `crates/magics/src/api/state.rs`: State structures and API state management
- `crates/magics/src/api/message.rs`: Message protocol definitions
- `crates/magics/src/api/zmq_server.rs`: ZeroMQ server implementation
- `crates/magics/src/api/extract.rs`: State extraction from simulation
- `crates/magics/src/api/factor_details.rs`: Factor graph details extraction
- `crates/magics/src/api/weights.rs`: Factor weight updates
- `crates/magics/src/api/reset.rs`: API state reset handling
- `crates/magics/src/api/state_utils.rs`: Utility functions for state creation (used by spawn/remove)
- `crates/magics/src/api/despawned_agents.rs`: Tracks agents removed via API for correct state reporting
- `python_api/magics_client.py`: Python client implementation (now includes spawn/remove, get_current_scenario)
- `python_api/stepping.py`: Example of stepping through simulation
- `python_api/agent_weights_example.py`: Example of setting per-agent factor weights

## Recent Changes
- Implemented per-agent factor weights system to allow individual agent customization
- Added getter and update methods to FactorGraph for per-agent weights
- Updated UI to use the new factor_weights() getter method
- Created a Python example script to demonstrate per-agent weight setting
- Documented the per-agent weights implementation in memory-bank
- Reorganized progress tracking to better reflect the current state of implementation
- Identified key next steps for API completion
- Planned a structured documentation system to improve API usability
- Verified the implementation status of all API components
- Identified TODOs in `crates/magics/src/api/extract.rs` for additional environment information
- **Implemented `SpawnAgent` and `RemoveAgent` API commands and corresponding Rust/Python logic.**
- **Implemented `GetCurrentScenario` API command and corresponding Rust/Python logic.**

## Next Steps

1. **Finalize API**: Finalizing the remaining aspects of the API implementation, reset environment, load environment.
2. **Extend confi**: Implement pause on load. 
3. **Finalize OpenAI Gym Integration**
   - Define observation and action spaces in `python_api/magics_gym/`
   - Implement proper reward calculation in `python_api/magics_gym/`
   - Create render methods in `python_api/magics_gym/`
   - Ensure proper state conversion between Rust and Python

## Documentation Structure Plan
```
crates/magics/src/api/
├── docs/
│   ├── README.md                  # Overview and navigation
│   ├── getting_started.md         # Quick start guide
│   ├── core_concepts.md           # Explanation of key concepts
│   ├── commands.md                # API commands reference
│   ├── data_structures/           # Detailed data structure documentation
│   │   ├── agent_state.md         # Agent state documentation
│   │   ├── environment_state.md   # Environment state documentation
│   │   ├── factor_graph.md        # Factor graph documentation
│   │   └── collision_info.md      # Collision information documentation
│   ├── examples/                  # Usage examples
│   │   ├── basic_usage.md         # Basic API usage
│   │   ├── stepping.md            # Stepping through simulation
│   │   └── weights.md             # Modifying factor weights
│   └── troubleshooting.md         # Common issues and solutions
└── API_DOCUMENTATION.md           # Main entry point (will link to docs folder)
```

## Implementation Details

### State Structures (`crates/magics/src/api/state.rs`)
- `AgentState`: Contains agent position, velocity, factor graph state, mission state, etc.
- `FactorGraphState`: Contains factor graph weights, variable/factor counts, message statistics
- `EnvironmentState`: Contains obstacle positions, boundaries, agent count, etc.
- `ApiState`: Main resource for managing API state, including agent states, environment state, weight requests, spawn/removal requests, and spawned agent IDs.

### Agent Spawning/Removal Implementation
- **Commands**: `SpawnAgent` and `RemoveAgent` added to `message.rs`.
- **State**: `ApiState` in `state.rs` extended with `SpawnParams` struct, `agent_spawn_requests`, `spawned_agent_ids`, and `agent_removal_requests` queues, plus methods to manage them.
- **Server**: `zmq_server.rs` updated to handle new commands, queue requests in `ApiState`, and wait for spawn completion using `spawned_agent_ids`.
- **Plugin**: `plugin.rs` updated with `handle_agent_spawn_requests` and `handle_agent_removal_requests` systems.
  - `handle_agent_spawn_requests` uses `Commands` to spawn a `RobotBundle` and associated components, mimicking `planner/spawner.rs`, and reports the new ID via `ApiState`.
  - `handle_agent_removal_requests` captures final state using `state_utils::create_agent_state`, adds it to `DespawnedAgentsTracker`, despawns the entity, and sends `RobotDespawned` event.
- **Client**: `magics_client.py` updated with `spawn_agent` and `remove_agent` methods.

### Agent-Based Factor Weights Implementation
The per-agent factor weights system allows each agent to have its own unique set of weights:

- **FactorWeights Structure**: Defined in `crates/magics/src/api/state.rs` with fields for dynamic, obstacle, interrobot, and tracking weights.
- **FactorGraph Methods**: Added getter and update methods in `crates/magics/src/factorgraph/factorgraph.rs` to manage weights.
- **API Integration**: The weight update system in `crates/magics/src/api/weights.rs` now processes agent-specific updates.
- **Python Client Method**: `set_factor_weights()` method accepts an optional agent_id parameter.
- **Example Script**: `python_api/agent_weights_example.py` demonstrates the complete workflow.

This system provides several benefits:
- Heterogeneous agent behavior through different navigation styles and priorities
- Role-based optimization for agents with different tasks
- Support for dynamic adaptation to changing conditions
- Better integration with reinforcement learning for agent-specific policies

### Message Protocol (`crates/magics/src/api/message.rs`)
- `Command`: Enum for all supported API operations (now includes `SpawnAgent`, `RemoveAgent`).
- `Request`: Message sent from client to server.
- `Response`: Message sent from server to client.
- `ResponseData`: Enum for response data types (now includes `SpawnedAgentId`).
- `SerializedAgentState`, `SerializedEnvironmentState`, etc.: Serialized versions of state structures

### ZeroMQ Server (`crates/magics/src/api/zmq_server.rs`)
- `ZmqServer`: Handles communication with clients via ZeroMQ
- `server_loop`: Main server loop for receiving and processing messages
- `handle_message`: Processes incoming messages and generates responses

### State Extraction (`crates/magics/src/api/extract.rs`)
- `extract_state`: Main system for extracting state from simulation
- Extracts agent states from robots query
- Extracts environment state from obstacles query
- Tracks collision information

### Factor Details Extraction (`crates/magics/src/api/factor_details.rs`)
- `extract_factor_details`: Extracts detailed information about factor graph components
- Extracts variable information
- Extracts obstacle factor information
- Extracts inter-robot factor information
- Extracts tracking factor information
- Extracts dynamic factor information

## Potential Research Questions
- How do different per-agent factor weight configurations affect collaborative behavior?
- Can agent specialization through custom weights improve overall system performance?
- What role does agent connectivity play in overall system performance?
- Can reinforcement learning discover optimal agent-specific weight configurations?
- How does the system scale with increasing agent counts under ML control?
- What emergent behaviors arise from learned weight parameters?

## Challenges
- Ensuring thread safety between ZMQ server and Bevy application
- Designing efficient serialization for state data
- Implementing proper error propagation from server to client
- Creating a clean Python API matching original functionality
- Balancing flexibility and complexity in the message protocol
- Defining appropriate observation and action spaces for RL
- Creating meaningful reward functions for path planning optimization
