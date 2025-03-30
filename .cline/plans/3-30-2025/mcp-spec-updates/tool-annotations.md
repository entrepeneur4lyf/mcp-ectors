# MCP Specification Update: Tool Annotations

**Version**: 1.0  
**Date**: 2025-03-30  
**Author**: Cline  
**Status**: Not Started  
**Priority**: P2

## Overview

This implementation plan addresses the Tool Annotations feature introduced in the MCP specification update from 2025-03-26. Tool Annotations provide enhanced metadata for tools, enabling better documentation, validation, and client-side capabilities. 

As detailed in the MCP architecture documentation, tools are model-controlled but should always have a human in the loop for security purposes. The annotations play a critical role in providing clients with information about tool behavior that can be used for user interfaces, security checks, and informed consent.

## Specification Details

The Tool Annotations feature adds the following capabilities:

- Richer metadata for tools beyond name, description, and schema
- Specific annotation properties to describe tool behavior:
  - `title`: Human-readable title for the tool
  - `readOnlyHint`: Indicates if the tool does not modify its environment
  - `destructiveHint`: Indicates if the tool performs destructive updates
  - `idempotentHint`: Indicates if calling the tool repeatedly with the same arguments has no additional effect
  - `openWorldHint`: Indicates if the tool interacts with an "open world" of external entities
- Explicit documentation that annotations are hints, not guarantees of behavior
- Standardized interface for annotation properties

## Progress Tracking

- **Overall Status**: Not Started
- **Estimated Duration**: 3 days
- **Expected Completion Date**: TBD
- **Related Phase**: Phase 4 (Router System Enhancements)

## Implementation Tasks

### 1. Annotation Data Model Implementation

**Status**: Not Started  
**Priority**: P2  
**Estimated Effort**: 1 day  
**Dependencies**: Error Handling Framework

**Description**:
Implement the data model for tool annotations according to the MCP specification.

**Acceptance Criteria**:
- Data structures for all annotation types (ToolAnnotations interface)
- Implementation of the five specific annotation properties:
  - `title`: String property for human-readable name
  - `readOnlyHint`: Boolean with default false
  - `destructiveHint`: Boolean with default true (when readOnlyHint is false)
  - `idempotentHint`: Boolean with default false (when readOnlyHint is false) 
  - `openWorldHint`: Boolean with default true
- Serialization/deserialization of annotations
- Validation logic for annotations
- Default values for optional annotations
- Backward compatibility with non-annotated tools

**Implementation Notes**:
- Use serde for serialization/deserialization
- Implement strong types for annotations
- Add validation using the validation framework
- Support optional annotations with sensible defaults
- Include documentation noting that annotations are hints, not guarantees

### 2. Router Integration

**Status**: Not Started  
**Priority**: P2  
**Estimated Effort**: 1 day  
**Dependencies**: Annotation Data Model Implementation

**Description**:
Integrate tool annotations with the router subsystem to expose annotations to clients.

**Acceptance Criteria**:
- Routers can provide annotations for their tools
- List tools request includes annotations
- Wasm routers can define annotations
- Annotations are properly validated
- Default annotations for tools without explicit annotations

**Implementation Notes**:
- Update router trait to support annotations
- Extend WASM interface for annotations
- Add annotation support to system router
- Implement automatic annotation generation for basic cases

### 3. Client-Side Annotation Handling

**Status**: Not Started  
**Priority**: P2  
**Estimated Effort**: 1 day  
**Dependencies**: Router Integration

**Description**:
Enhance the client interface to consume and utilize tool annotations.

**Acceptance Criteria**:
- Client can retrieve and parse tool annotations
- Annotation information available to client applications
- Support for annotation-based validation
- Helper functions for common annotation use cases
- Proper error handling for annotation-related operations

**Implementation Notes**:
- Implement annotation helpers in the client library
- Add validation based on annotations
- Create examples of annotation usage
- Document annotation-based capabilities

## Dependencies

- Error Handling Framework (Phase 1)
- Router System Enhancements (Phase 4)

## Integration with Existing Components

The Tool Annotations will be integrated with:

- **Router Subsystem**: To provide annotations for tools
- **Client Interface**: To consume and utilize annotations
- **WASM Interface**: To allow WASM routers to define annotations
- **Documentation Generation**: To include annotation information in docs

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Annotation complexity overwhelming users | Medium | Medium | Provide sensible defaults, documentation, and examples |
| Backward compatibility issues | Medium | Low | Ensure all annotations are optional, maintain non-annotated path |
| Performance impact of annotation processing | Low | Low | Optimize annotation processing, consider caching strategies |
| Annotation validation overhead | Low | Low | Efficient validation implementation, optional validation |
| Security misinterpretation | High | Medium | Clearly document that annotations are hints, not guarantees; implement proper security checks |
| User consent challenges | High | Medium | Provide clear UI guidelines for using annotations to inform users about tool behavior |

## Acceptance Criteria

The Tool Annotations implementation will be considered complete when:

1. All MCP specification requirements for tool annotations are implemented
2. Routers can define and expose annotations for their tools
3. Clients can consume and utilize annotations
4. Comprehensive tests demonstrate correct behavior
5. Documentation explains how to use annotations for both router developers and clients

## Implementation Notes

- Focus on the most valuable annotations first
- Provide automatic annotation generation where possible
- Ensure backward compatibility with existing routers and clients
- Create comprehensive examples of annotation usage
- Document best practices for annotation definition
- Emphasize that annotations are hints and not security guarantees
- Provide sample UI elements that use annotations to inform users about tool behavior
- Align with the MCP architecture design principle that "Servers should be extremely easy to build"
- Consider the user interaction model where "there SHOULD always be a human in the loop with the ability to deny tool invocations"
