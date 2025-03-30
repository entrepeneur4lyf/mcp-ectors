use thiserror::Error;

/// Custom error type for handling transport layer errors.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Represents network-related errors (e.g. connection issues).
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Represents protocol errors (e.g. malformed messages).
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Represents internal server errors.
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Represents missing or incorrect configuration.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Represents IO errors
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Represents serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerdeError(String)
}

impl TransportError {
    /// Creates a new `TransportError::NetworkError`.
    pub fn network_error(msg: impl Into<String>) -> Self {
        TransportError::NetworkError(msg.into())
    }

    /// Creates a new `TransportError::ProtocolError`.
    pub fn protocol_error(msg: impl Into<String>) -> Self {
        TransportError::ProtocolError(msg.into())
    }

    /// Creates a new `TransportError::InternalError`.
    pub fn internal_error(msg: impl Into<String>) -> Self {
        TransportError::InternalError(msg.into())
    }

    /// Creates a new `TransportError::ConfigurationError`.
    pub fn configuration_error(msg: impl Into<String>) -> Self {
        TransportError::ConfigurationError(msg.into())
    }

    /// Creates a new `TransportError::SerdeError`.
    pub fn serde_error(msg: impl Into<String>) -> Self {
        TransportError::SerdeError(msg.into())
    }
}

// Helper function to quickly convert errors to a `TransportError`.
impl From<&str> for TransportError {
    fn from(s: &str) -> Self {
        TransportError::InternalError(s.to_string())
    }
}

impl From<String> for TransportError {
    fn from(s: String) -> Self {
        TransportError::InternalError(s)
    }
}
