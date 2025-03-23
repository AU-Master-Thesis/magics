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
   - Step-based simulation control
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
    end
    
    Gym -- "step(), reset()" --> PyClient
    PyClient -- "ZMQ REQ" --> ZmqServer
    ZmqServer -- "ZMQ REP" --> PyClient
    ZmqServer -- "Accesses" --> ApiState
    ApiPlugin -- "Manages" --> ZmqServer
    ApiPlugin -- "Controls" --> ApiState
    ApiState -- "Synchronizes" --> Sim
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
    StateExtraction --> AgentState
    StateExtraction --> EnvironmentState
```

## API Data Flow
```mermaid
graph LR
    subgraph Python
        Step[Step Command]
        Weights[Factor Weights]
        Observation[State Observation]
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
        Simulation[Simulation Loop]
    end
    
    Step --> ReqSocket
    ReqSocket --> RepSocket
    RepSocket --> ZmqThread
    ZmqThread --> ApiState
    ApiState --> Control
    Control --> Simulation
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
```

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
