# Active Context

## Current Development Focus
The current development focus is on three main areas:

1. **API Feature Fix**: Ensuring the ApiPlugin is only loaded when the "api" feature is enabled, preventing auto-pause behavior when not using the API.

2. **Environment Loading Investigation**: Diagnosing why environments aren't loading properly in scenarios, despite having the correct file paths.

3. **Scenario Organization**: Renaming scenario folders to PascalCase (no spaces) for better cross-platform compatibility and updating all references.

## Recent Changes
- Fixed API plugin conditional loading in main.rs to only add the plugin when the "api" feature is enabled
- Updated scenario config files to point to their own environment and formation files
- Restructured progress.md to use a cleaner checklist format
- Updated activeContext.md to reflect current priorities

## Next Steps
1. **API Feature Fix**
   - Test running without the API feature to ensure it doesn't auto-pause
   - Verify that the ZMQ server is only started when the API feature is enabled

2. **Environment Loading Investigation**
   - Check environment file contents for correctness
   - Verify rendering systems are working properly
   - Test with a simple environment to isolate the issue
   - Investigate potential issues with the environment loading code

3. **Scenario Folder Renaming**
   - Create a systematic approach to rename all folders from spaces to PascalCase
   - Update all config files to reference the new paths
   - Test to ensure all scenarios load correctly after renaming

## Potential Research Questions
- How do different factor graph weight configurations affect path planning efficiency?
- What role does agent connectivity play in overall system performance?
- Can reinforcement learning discover optimal weight configurations?
- How does the system scale with increasing agent counts under ML control?
- What emergent behaviors arise from learned weight parameters?

## Challenges
- Ensuring thread safety between ZMQ server and Bevy application
- Managing appropriate timeout handling for step commands
- Designing efficient serialization for state data
- Implementing proper error propagation from server to client
- Creating a clean Python API matching original functionality
- Balancing flexibility and complexity in the message protocol
- Diagnosing environment loading issues in scenarios
- Ensuring cross-platform compatibility with scenario paths
