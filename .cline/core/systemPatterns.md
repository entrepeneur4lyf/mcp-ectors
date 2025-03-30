# System Patterns: mcp-ectors

## Architectural Overview

The mcp-ectors system is built on several key architectural patterns that enable its high performance, security, and extensibility:

## 1. Actor Model Pattern

The core architectural pattern of mcp-ectors is the **Actor Model**:

- **Isolated State**: Each actor maintains its own state and can only modify that state
- **Message-Driven Communication**: Actors communicate exclusively through message passing
- **Concurrency Control**: The actor model naturally handles concurrency without shared state
- **Fault Isolation**: Failures in one actor don't directly affect others

Key actors in the system include:
- **Router Actor**: Manages MCP routers and their lifecycle
- **Transport Actor**: Handles the communication transport layer (SSE, etc.)
- **Topic Registry Actor**: Manages subscriptions to topics
- **System Router**: Provides core system functionality

## 2. Registry Pattern

The system uses registries to manage collections of related components:

- **Router Registry**: Manages the registration and retrieval of routers
- **Client Registry**: Tracks active client connections
- **Topic Registry**: Manages topic subscriptions for notifications

This pattern allows for dynamic registration and discovery of components.

## 3. Service Manager Pattern

The **Router Service Manager** implements the service manager pattern:

- **Dynamic Service Registration**: Allows runtime registration of new routers
- **Service Discovery**: Provides mechanisms to find and use registered services
- **Lifecycle Management**: Handles initialization and shutdown of services

## 4. WebAssembly Containment Pattern

MCP routers run in isolated WebAssembly environments:

- **Sandboxed Execution**: Routers operate in a contained environment
- **Defined Interface**: Communication through well-defined WIT interfaces
- **Resource Limitations**: Controlled access to system resources
- **Cross-Platform Consistency**: Same behavior across different host environments

## 5. Message-Based Communication Pattern

All components communicate through well-defined messages:

- **Transport Messages**: For managing transport connections
- **Router Messages**: For interactions with routers
- **Client Messages**: For client session management
- **MCP Protocol Messages**: Following the MCP specification

## 6. Transport Abstraction Pattern

The system abstracts away transport details:

- **Transport Actor Trait**: Common interface for all transports
- **Specific Implementations**: SSE, stdio, WASI implementations
- **Message Transformation**: Convert between transport-specific and internal messages

## 7. Component-Based Extension Pattern

New capabilities are added through components rather than modifying the core:

- **New Router Implementation**: Adding capabilities through new routers
- **Transport Extensions**: Supporting new transport mechanisms
- **Protocol Extensions**: Extending the MCP protocol with new messages

## System Interaction Flow

1. Client connects via a transport (e.g., SSE)
2. Transport Actor receives the connection and creates a Client Session
3. Client requests are transformed into internal messages
4. Messages are routed to appropriate Router Actors
5. Router Actors process requests and return responses
6. Responses are sent back through the Transport Actor to the Client

## Architectural Constraints

- **Wasm Security Boundary**: Routers can only interact through the defined WIT interface
- **Message-Only Communication**: No direct method calls between components
- **Actor Isolation**: Actors maintain and modify only their own state
- **Standardized Protocol**: All interactions follow the MCP protocol specification
