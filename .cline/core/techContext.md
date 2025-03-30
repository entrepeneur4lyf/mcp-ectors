# Technology Context: mcp-ectors

## Technology Stack

### Core Technologies
- **Rust**: Primary programming language (edition 2021)
- **Actix**: Actor framework for concurrency (v0.13.5)
- **WebAssembly (Wasm)**: For secure, isolated execution of routers
- **WASIX**: Extended WebAssembly Interface for additional capabilities

### Web & Transport
- **Actix Web**: Web framework for HTTP/SSE transports (v4.9.0)
- **Actix Codec**: For stream encoding/decoding (v0.5.2)
- **TLS**: Via rustls (v0.23.25) for secure communications

### Runtime & Execution
- **Wasmtime**: WebAssembly runtime (v31.0.0)
- **Tokio**: Asynchronous runtime (v1.44.1)
- **Actix-rt**: Actix runtime (v2.10.0)

### Data Handling
- **Serde**: Serialization/deserialization framework (v1.0.218)
- **Serde JSON**: JSON implementation for Serde (v1.0.140)
- **Schemars**: JSON Schema generation (v0.8.22)

### Utilities
- **Tracing**: Logging and diagnostics (v0.1.41)
- **Anyhow/Thiserror**: Error handling (v1.0.97/v2.0.12)
- **Chrono**: Date and time handling (v0.4.40)
- **Clap**: Command-line argument parsing (v4.5.32)
- **Notify**: Filesystem notification (v8.0.0)

### Protocol
- **MCP Spec**: Model Context Protocol specification (v0.1.0)

## Development Environment Setup

### Prerequisites
- Rust toolchain (stable, minimum 1.63)
- Cargo package manager
- Git for version control

### Build Process
```bash
# Clone the repository
git clone https://github.com/mcp-ectors/mcp-ectors.git
cd mcp-ectors

# Build the project
cargo build

# Run in development mode
cargo run

# Build for production
cargo build --release
```

### Project Structure
- **src/**: Source code
  - **client/**: Client session management
  - **examples/**: Example routers (Counter, HelloWorld)
  - **mcp/**: MCP protocol implementation
  - **messages/**: Message definitions
  - **router/**: Router implementation and management
  - **transport/**: Transport layer implementations
  - **utils/**: Utility functions

### MCP Router Development
New routers should:
1. Implement the `Guest` trait from the WIT interface
2. Be compiled to Wasm with wasm32-wasi target
3. Implement required methods for tools, resources, or prompts
4. Be placed in the wasm directory for auto-loading

## Deployment Requirements

### System Requirements
- Modern Linux, macOS, or Windows server
- Sufficient RAM for Wasm execution (minimum 2GB recommended)
- Storage for logs and Wasm modules

### Configuration
- Environment variables or configuration file for:
  - Log level and directory
  - Transport settings (ports, TLS)
  - Wasm directory path

### Execution
```bash
# Start the server with custom wasm path
mcp-ectors start --wasm_path /path/to/wasm
```

## Testing Framework
- Uses standard Rust testing framework
- Includes mock routers for testing
- Integration tests with mcp-client

## Dependencies Management
- All dependencies are managed through Cargo
- Version constraints specified in Cargo.toml
- Development dependencies separated from runtime dependencies
