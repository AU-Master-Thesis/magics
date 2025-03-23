# System Patterns and Architecture

## Overall Architecture
Magics uses a modular, component-based architecture centered around the Factor Graph concept, now extended with a ZeroMQ-based API layer for external control and reinforcement learning integration.

## Core Architectural Components
1. Factor Graph
   - Represents probabilistic relationships between variables
   - Enables decentralized path planning
   - Supports multiple factor types:
     * Dynamic Factors
     * Obstacle Factors
     * Inter-robot Factors
     * Tracking Factors

2. Message Passing Mechanism
   - Implements Gaussian Belief Propagation
   - Bidirectional message exchange between variables and factors
   - Probabilistic belief updates

3. Simulation Framework
   - Bevy ECS for entity management
   - Modular plugin system
   - Configurable simulation parameters

4. ZeroMQ API Layer
   - REQ-REP socket pattern
   - JSON message serialization
   - Thread-safe state extraction
   - FixedUpdate-based step control
   - Factor graph weight modification

## API Architecture
```mermaid
graph TB
    subgraph Python
        Gym[OpenAI Gym Environment]
        PyClient[Python ZMQ Client]
    end
    
    subgraph Rust
        ZmqServer[ZMQ Server Thread]
        ApiPlugin[API Plugin]
        ApiState[API State Resource]
        Sim[Simulation Systems]
        FactorGraph[Factor Graph]
        FixedUpdate[FixedUpdate System]
    end
    
    Gym -- "step(), reset()" --> PyClient
    PyClient -- "ZMQ REQ" --> ZmqServer
    ZmqServer -- "ZMQ REP" --> PyClient
    ZmqServer -- "Accesses" --> ApiState
    ApiPlugin -- "Manages" --> ZmqServer
    ApiPlugin -- "Controls" --> ApiState
    ApiState -- "Synchronizes" --> Sim
    ApiState -- "Counts" --> FixedUpdate
    FixedUpdate -- "Advances" --> Sim
    Sim -- "Updates" --> FactorGraph
```

## Design Patterns
- Factor Graph Pattern
- Message Passing Pattern
- Probabilistic Inference
- Dependency Injection
- Plugin Architecture
- Actor Model (via ZeroMQ)
- Observer Pattern (for state extraction)
- Command Pattern (for step control)
- Strategy Pattern (for factor weight application)

## Key Technical Decisions
1. Language and Performance
   - Rust for high performance and memory safety
   - Minimal runtime overhead
   - Zero-cost abstractions

2. Probabilistic Modeling
   - Gaussian Belief Propagation algorithm
   - Probabilistic representation of robot states
   - Uncertainty-aware path planning

3. Modularity and Extensibility
   - Crate-based project structure
   - Flexible factor and variable definitions
   - Easy scenario configuration
   - Plugin-based API extension

4. API Design
   - ZeroMQ for language-agnostic communication
   - REQ-REP pattern for synchronous operations
   - JSON for human-readable message format
   - Thread-safe shared state
   - FixedUpdate-based stepping for consistent time advancement

## Component Relationships
```mermaid
graph TD
    FactorGraph --> Variables
    FactorGraph --> Factors
    Variables --> MessagePassing
    Factors --> MessagePassing
    MessagePassing --> BeliefUpdates
    BeliefUpdates --> PathPlanning
    PathPlanning --> RobotNavigation
    ApiPlugin --> ZmqServer
    ZmqServer --> ApiState
    ApiPlugin --> PausePlay
    ApiPlugin --> StateExtraction
    ApiPlugin --> WeightModification
    ApiPlugin --> FixedUpdateMonitoring
    StateExtraction --> AgentState
    StateExtraction --> EnvironmentState
    FixedUpdateMonitoring --> StepCompletion
```

## API Data Flow
```mermaid
graph LR
    subgraph Python
        Step[Step Command]
        Weights[Factor Weights]
        Observation[State Observation]
        Config[Config Access]
    end
    
    subgraph ZMQ
        ReqSocket[REQ Socket]
        RepSocket[REP Socket]
    end
    
    subgraph Rust
        ZmqThread[ZMQ Server Thread]
        ApiState[API State]
        Control[Control System]
        Extraction[State Extraction]
        Modification[Weight Modification]
        FixedUpdate[FixedUpdate System]
        ConfigAccess[Config Access]
    end
    
    Step --> ReqSocket
    ReqSocket --> RepSocket
    RepSocket --> ZmqThread
    ZmqThread --> ApiState
    ApiState --> Control
    Control --> FixedUpdate
    FixedUpdate --> Simulation
    Simulation --> Extraction
    Extraction --> ApiState
    ZmqThread --> ApiState
    ApiState --> RepSocket
    RepSocket --> ReqSocket
    ReqSocket --> Observation
    
    Weights --> ReqSocket
    ReqSocket --> RepSocket
    RepSocket --> ZmqThread
    ZmqThread --> ApiState
    ApiState --> Modification
    Modification --> Simulation
    
    Config --> ReqSocket
    ReqSocket --> RepSocket
    RepSocket --> ZmqThread
    ZmqThread --> ConfigAccess
    ConfigAccess --> ApiState
    ApiState --> RepSocket
    RepSocket --> ReqSocket
    ReqSocket --> Config
```

## Stepping Mechanism
The API now uses a simplified stepping approach based on FixedUpdate counting:

1. When a step is requested:
   - The API state sets the number of iterations to run
   - The simulation is unpaused
   - FixedUpdate iterations are counted

2. During each FixedUpdate:
   - The iteration counter is decremented
   - The simulation advances by one fixed timestep
   - State is extracted for API access

3. When iterations are complete:
   - The simulation is paused
   - The step is marked as completed
   - The ZMQ server responds to the client

This approach ensures consistent time advancement regardless of frame rate or system performance.

## Computational Complexity
- Message Passing: O(n) where n is number of factors/variables
- Path Planning: O(log n)
- Collision Avoidance: O(m²) where m is number of robots
- State Extraction: O(m) where m is number of agents
- API Communication: O(1) per request

## Constraints and Considerations
- Real-time performance requirements
- Probabilistic uncertainty management
- Scalability across different scenario complexities
- Thread safety for ZeroMQ server thread
- Consistent state representation for ML algorithms
- Timeout handling for synchronous operations
- Consistent time advancement for reinforcement learning
