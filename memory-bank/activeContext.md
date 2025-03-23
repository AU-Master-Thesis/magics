# Active Context

## Current Development Focus
The current development focus is on three main areas:

1. **Expanding API Functionality**: Enhancing the API to expose all agent and environment data needed for reinforcement learning, and adding access to configuration parameters like simulation Hz.

2. **OpenAI Gym Integration**: Completing the Python API integration with OpenAI Gym, including observation and action spaces, step/reset methods, and reward calculation.

3. **API Data Flow Optimization**: Ensuring efficient and accurate data extraction and serialization between the Rust simulation and Python client.

## Recent Changes
- Simplified the API stepping approach by removing manual stepping and standardizing on FixedUpdate-based stepping
- Fixed virtual time advancement by using FixedUpdate counting for consistent step timing
- Renamed scenario folders to PascalCase (no spaces) for better cross-platform compatibility
- Updated all references in config files to match the new scenario folder names
- Fixed environment loading issues in scenarios

## Next Steps
1. **Expand API Data Extraction**
   - Ensure all agent data is properly exposed through the API
   - Verify environment data extraction is complete and accurate
   - Add any missing state information needed for ML algorithms

2. **Add Config Hz Access**
   - Implement API endpoint to get the simulation Hz
   - Add functionality to set the Hz if needed
   - Update message protocol to support these operations

3. **Complete OpenAI Gym Integration**
   - Implement observation and action spaces
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
