use thiserror::Error;

/// Root error type for all MCP-Ectors errors.
/// This type can be used to consolidate errors from different subsystems
/// into a single error type that can be returned from public APIs.
#[derive(Debug, Error)]
pub enum McpError {
    /// Errors related to transport layer operations
    #[error("Transport error: {0}")]
    Transport(#[from] crate::transport::transport_error::TransportError),

    /// Errors related to message processing
    #[error("Message error: {0}")]
    Message(String),

    /// Errors related to router operations
    #[error("Router error: {0}")]
    Router(String),

    /// Errors related to client operations
    #[error("Client error: {0}")]
    Client(String),

    /// Errors related to configuration
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Errors related to WASM operations
    #[error("WASM error: {0}")]
    Wasm(String),

    /// Errors related to JSON-RPC processing
    #[error("JSON-RPC error: {0}")]
    JsonRpc(String),

    /// Catch-all for internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias with McpError as the error type
pub type Result<T> = std::result::Result<T, McpError>;

/// Context extension trait for Result types
/// Allows adding context to errors with the `.context()` method
pub trait ResultExt<T, E> {
    /// Add context to an error
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static;
}

/// Implementation of ResultExt for any Result type
impl<T, E> ResultExt<T, E> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T>
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            let context_str = format!("{}: {}", context, err);
            McpError::Internal(context_str)
        })
    }
}

/// Helper function to create an internal error
pub fn internal_err<S: Into<String>>(message: S) -> McpError {
    McpError::Internal(message.into())
}

/// Helper function to create a configuration error
pub fn config_err<S: Into<String>>(message: S) -> McpError {
    McpError::Configuration(message.into())
}

/// Helper function to create a transport error
pub fn transport_err<S: Into<String>>(message: S) -> McpError {
    McpError::Transport(crate::transport::transport_error::TransportError::internal_error(message))
}

/// Helper function to create a router error
pub fn router_err<S: Into<String>>(message: S) -> McpError {
    McpError::Router(message.into())
}

/// Helper function to create a client error
pub fn client_err<S: Into<String>>(message: S) -> McpError {
    McpError::Client(message.into())
}

/// Helper function to create a JSON-RPC error
pub fn jsonrpc_err<S: Into<String>>(message: S) -> McpError {
    McpError::JsonRpc(message.into())
}

/// Helper function to create a WASM error
pub fn wasm_err<S: Into<String>>(message: S) -> McpError {
    McpError::Wasm(message.into())
}
