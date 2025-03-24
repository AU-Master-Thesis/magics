# Active Context

## Current Development Focus
The current development focus is on implementing API expansions to support reinforcement learning research:

1. **Expanding State Structures**: Enhancing the AgentState, FactorGraphState, and EnvironmentState structures to expose comprehensive agent and environment data needed for reinforcement learning.

2. **Implementing Serialization Protocol**: Updating the message protocol to support serialization of expanded state structures between Rust and Python.

3. **Enhancing State Extraction**: Modifying the extract_state function to pull detailed agent and environment information from Bevy ECS components.

4. **Updating Python Client**: Enhancing the Python client to handle the expanded state information and provide helper methods for accessing specific data.

## Recent Changes
- Implemented the core state structure expansions for AgentState, FactorGraphState, and EnvironmentState
- Added Default implementations for all new state structures to simplify instantiation
- Created helper methods in ApiState for convenient access to entity and environment data
- Added config Hz access via API with proper updating of both Config and Time<Fixed> resources
- Enhanced plugin.rs to use default-initialized structures as starting points for data extraction
- Added placeholders and TODOs in extract_state for full implementation of data extraction

## Next Steps
1. **Complete API Data Extraction**
   - Extract actual data for all new state fields from ECS components
   - Connect factor graph details to actual factor graph data
   - Implement extraction of mission state and planning strategy
   - Extract detailed environment information (SDF resolution, density maps)

2. **Implement Message Protocol Updates**
   - Create serialized versions of all new state structures
   - Ensure JSON compatibility for all types
   - Implement From trait conversions for serialization/deserialization

3. **Update Python Client**
   - Enhance MagicsClient to handle expanded state information
   - Create helper methods for accessing specific data elements
   - Implement NumPy array conversions for numerical data
   - Add proper type hints and documentation

4. **Complete OpenAI Gym Integration**
   - Implement observation and action spaces based on expanded state
   - Create step, reset, and render methods
   - Add reward calculation
   - Ensure proper state conversion between Rust and Python

## Potential Research Questions
- How do different factor graph weight configurations affect path planning efficiency?
- What role does agent connectivity play in overall system performance?
- Can reinforcement learning discover optimal weight configurations?
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
