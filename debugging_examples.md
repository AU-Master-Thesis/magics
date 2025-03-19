# Debugging Rust with VSCode

This document provides instructions for debugging your Rust application in VSCode, specifically for the headless mode implementation in the Magics project.

## Setup and Configuration

1. The `.vscode/launch.json` file has already been set up with two debug configurations:
   - "Debug Headless (Circle Experiment)"
   - "Debug Headless (Solo GP)"

2. These configurations use LLDB to debug the Rust code.

## Setting Breakpoints

To set breakpoints in VSCode, you can:

1. Click in the gutter (left margin) next to the line number where you want to set a breakpoint
2. Or press F9 when your cursor is on the line where you want to set a breakpoint

Good places to set breakpoints to verify headless mode is working:

- In `crates/magics-cli/src/lib.rs` around line 95-100 where the simulation is created
- In the simulation step loop around line 110 to inspect agent states
- In `crates/magics-core/src/environment.rs` to see environment updates
- In `crates/magics-core/src/simulation.rs` in the `step()` method

## Running the Debugger

1. Open the Debug view in VSCode (click the bug icon in the left sidebar or press Ctrl+Shift+D)
2. Select one of the debug configurations from the dropdown at the top
3. Click the green play button or press F5

## While Debugging

- The debugger will stop at your breakpoints
- You can inspect variables in the "VARIABLES" panel
- Use the debug controls to:
  - Continue (F5)
  - Step Over (F10)
  - Step Into (F11)
  - Step Out (Shift+F11)

## Inspecting Variables

When stopped at a breakpoint, you can:

1. Hover over variables to see their values
2. Use the "WATCH" panel to track specific variables
3. Use the Debug Console (Ctrl+Shift+Y) to evaluate expressions

## Recommended Debug Workflow

1. Set breakpoints in key places:
   - At the start of the headless mode function
   - Where the environment is created
   - In the simulation loop
   - At the end to inspect final state

2. Launch the debugger with one of the configured scenarios

3. As execution stops at each breakpoint, verify:
   - Environment is properly loaded
   - Agents are initialized correctly
   - Simulation steps are processed
   - Agent states are updated between steps

This approach will help you verify that the headless mode is actually running the simulation correctly, even without visual feedback.
