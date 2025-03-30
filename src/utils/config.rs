use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use config::{Config as ConfigLib, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};

use crate::utils::error::{McpError, Result, config_err};

/// Validation trait for configuration types
pub trait Validate {
    /// Validates the configuration and returns any errors
    fn validate(&self) -> Result<()>;
}

/// Log levels for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)] // Added Default derive
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default] // Mark Info as the default variant
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

impl FromStr for LogLevel {
    type Err = McpError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(config_err(format!("Invalid log level: {}", s))),
        }
    }
}

// Removed manual Default impl

/// Configuration for the logging system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// The log level for the application
    #[serde(default)]
    pub level: LogLevel,

    /// Whether to include timestamps in logs
    #[serde(default = "default_true")]
    pub timestamps: bool,

    /// Log file path, if logging to a file
    #[serde(default)]
    pub file: Option<PathBuf>,

    /// Whether to output logs to stdout
    #[serde(default = "default_true")]
    pub stdout: bool,

    /// Format for logs (json or plain)
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            timestamps: true,
            file: None,
            stdout: true,
            format: "plain".to_string(),
        }
    }
}

impl Validate for LoggingConfig {
    fn validate(&self) -> Result<()> {
        // Validate log format
        if self.format != "plain" && self.format != "json" {
            return Err(config_err(format!("Invalid log format: {}", self.format)));
        }

        // Validate log file path if specified
        if let Some(path) = &self.file {
            let parent = path.parent().ok_or_else(|| {
                config_err(format!("Invalid log file path: {}", path.display()))
            })?;

            if !parent.exists() {
                return Err(config_err(format!(
                    "Log file directory does not exist: {}",
                    parent.display()
                )));
            }
        }

        // Ensure at least one output is enabled
        if !self.stdout && self.file.is_none() {
            return Err(config_err(
                "At least one log output (stdout or file) must be enabled"
            ));
        }

        Ok(())
    }
}

/// Configuration for the transport layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Bind address for the HTTP server
    #[serde(default = "default_bind_address")]
    pub bind_address: String,

    /// Port for the HTTP server
    #[serde(default = "default_port")]
    pub port: u16,

    /// TLS certificate path
    #[serde(default)]
    pub tls_cert: Option<PathBuf>,

    /// TLS key path
    #[serde(default)]
    pub tls_key: Option<PathBuf>,

    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Maximum request size in bytes
    #[serde(default = "default_max_request_size")]
    pub max_request_size: usize,

    /// Keep-alive timeout in seconds
    #[serde(default = "default_keep_alive")]
    pub keep_alive_secs: u64,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            bind_address: default_bind_address(),
            port: default_port(),
            tls_cert: None,
            tls_key: None,
            timeout_secs: default_timeout(),
            max_request_size: default_max_request_size(),
            keep_alive_secs: default_keep_alive(),
        }
    }
}

impl Validate for TransportConfig {
    fn validate(&self) -> Result<()> {
        // Validate TLS configuration
        if (self.tls_cert.is_some() && self.tls_key.is_none()) ||
           (self.tls_cert.is_none() && self.tls_key.is_some()) {
            return Err(config_err(
                "Both TLS certificate and key must be specified together"
            ));
        }

        // Validate TLS files exist if specified
        if let Some(cert_path) = &self.tls_cert {
            if !cert_path.exists() {
                return Err(config_err(format!(
                    "TLS certificate file does not exist: {}",
                    cert_path.display()
                )));
            }
        }

        if let Some(key_path) = &self.tls_key {
            if !key_path.exists() {
                return Err(config_err(format!(
                    "TLS key file does not exist: {}",
                    key_path.display()
                )));
            }
        }

        // Validate timeout values
        if self.timeout_secs == 0 {
            return Err(config_err("Timeout must be greater than 0"));
        }

        if self.max_request_size == 0 {
            return Err(config_err("Maximum request size must be greater than 0"));
        }

        Ok(())
    }
}

/// Configuration for the router system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Directory to scan for WASM router modules
    #[serde(default)]
    pub wasm_dir: Option<PathBuf>,

    /// Whether to watch the WASM directory for changes
    #[serde(default = "default_false")]
    pub watch_wasm_dir: bool,

    /// Memory limit for WASM modules in MB
    #[serde(default = "default_wasm_memory_limit")]
    pub wasm_memory_limit_mb: usize,

    /// Maximum number of instances per router
    #[serde(default = "default_max_instances")]
    pub max_instances: usize,

    /// Whether to preload routers at startup
    #[serde(default = "default_true")]
    pub preload_routers: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            wasm_dir: None,
            watch_wasm_dir: false,
            wasm_memory_limit_mb: default_wasm_memory_limit(),
            max_instances: default_max_instances(),
            preload_routers: true,
        }
    }
}

impl Validate for RouterConfig {
    fn validate(&self) -> Result<()> {
        // Validate WASM directory if specified
        if let Some(dir) = &self.wasm_dir {
            if !dir.exists() {
                return Err(config_err(format!(
                    "WASM directory does not exist: {}",
                    dir.display()
                )));
            }

            if !dir.is_dir() {
                return Err(config_err(format!(
                    "WASM path is not a directory: {}",
                    dir.display()
                )));
            }
        }

        // Validate memory limit
        if self.wasm_memory_limit_mb == 0 {
            return Err(config_err("WASM memory limit must be greater than 0"));
        }

        // Validate max instances
        if self.max_instances == 0 {
            return Err(config_err("Maximum instances must be greater than 0"));
        }

        Ok(())
    }
}

/// Configuration for security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether authentication is enabled
    #[serde(default = "default_false")]
    pub auth_enabled: bool,

    /// JWT secret for authentication
    #[serde(default)]
    pub jwt_secret: Option<String>,

    /// API keys mapped to roles
    #[serde(default)]
    pub api_keys: HashMap<String, String>,

    /// Allowed origins for CORS
    #[serde(default)]
    pub allowed_origins: Vec<String>,

    /// Whether to enforce HTTPS
    #[serde(default = "default_false")]
    pub enforce_https: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth_enabled: false,
            jwt_secret: None,
            api_keys: HashMap::new(),
            allowed_origins: vec!["*".to_string()],
            enforce_https: false,
        }
    }
}

impl Validate for SecurityConfig {
    fn validate(&self) -> Result<()> {
        // If auth is enabled, JWT secret is required
        if self.auth_enabled && self.jwt_secret.is_none() && self.api_keys.is_empty() {
            return Err(config_err(
                "When authentication is enabled, either JWT secret or API keys must be provided"
            ));
        }

        // Validate any JWT secret is at least 32 chars
        if let Some(secret) = &self.jwt_secret {
            if secret.len() < 32 {
                return Err(config_err(
                    "JWT secret must be at least 32 characters long"
                ));
            }
        }

        // Validate HTTPS enforcement with TLS
        if self.enforce_https {
            // Note: we can't validate TLS cert/key here since this config doesn't have
            // access to the transport config. This will be handled at a higher level.
        }

        Ok(())
    }
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    #[serde(default = "default_app_name")]
    pub name: String,

    /// Environment (development, production, testing)
    #[serde(default = "default_environment")]
    pub environment: String,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Transport configuration
    #[serde(default)]
    pub transport: TransportConfig,

    /// Router configuration
    #[serde(default)]
    pub router: RouterConfig,

    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,

    /// Additional custom settings
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            environment: default_environment(),
            logging: LoggingConfig::default(),
            transport: TransportConfig::default(),
            router: RouterConfig::default(),
            security: SecurityConfig::default(),
            custom: HashMap::new(),
        }
    }
}

impl Validate for AppConfig {
    fn validate(&self) -> Result<()> {
        // Validate environment
        let valid_envs = ["development", "production", "testing"];
        if !valid_envs.contains(&self.environment.as_str()) {
            return Err(config_err(format!(
                "Invalid environment: {}. Must be one of: development, production, testing",
                self.environment
            )));
        }

        // Validate subsystem configurations
        self.logging.validate()
            .map_err(|e| config_err(format!("Invalid logging configuration: {}", e)))?;

        self.transport.validate()
            .map_err(|e| config_err(format!("Invalid transport configuration: {}", e)))?;

        self.router.validate()
            .map_err(|e| config_err(format!("Invalid router configuration: {}", e)))?;

        self.security.validate()
            .map_err(|e| config_err(format!("Invalid security configuration: {}", e)))?;

        // Cross-validation between security and transport
        if self.security.enforce_https &&
           (self.transport.tls_cert.is_none() || self.transport.tls_key.is_none()) {
            return Err(config_err(
                "HTTPS enforcement requires TLS certificate and key to be configured"
            ));
        }

        Ok(())
    }
}

/// Configuration builder
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config_dir: Option<PathBuf>,
    config_file: Option<PathBuf>,
    env_prefix: Option<String>,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the configuration directory
    pub fn with_config_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.config_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Set the configuration file
    pub fn with_config_file<P: AsRef<Path>>(mut self, file: P) -> Self {
        self.config_file = Some(file.as_ref().to_path_buf());
        self
    }

    /// Set the environment variable prefix
    pub fn with_env_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<Arc<AppConfig>> {
        let mut builder = ConfigLib::builder();

        // Add default values
        builder = builder.add_source(ConfigLib::try_from(&AppConfig::default())
            .map_err(|e| config_err(format!("Failed to load default config: {}", e)))?);

        // Add configuration directory if specified
        if let Some(dir) = self.config_dir {
            if dir.exists() && dir.is_dir() {
                // Look for config files in common formats
                for ext in &["toml", "json", "yaml", "yml"] {
                    let config_path = dir.join(format!("config.{}", ext));
                    if config_path.exists() {
                        builder = builder.add_source(File::from(config_path));
                        break;
                    }
                }
            }
        }

        // Add specific config file if specified
        if let Some(file) = self.config_file {
            if file.exists() {
                // Determine format from extension
                let format = match file.extension().and_then(|ext| ext.to_str()) {
                    Some("json") => FileFormat::Json,
                    Some("toml") => FileFormat::Toml,
                    Some("yaml") | Some("yml") => FileFormat::Yaml,
                    _ => return Err(config_err(format!(
                        "Unsupported config file format: {}",
                        file.display()
                    ))),
                };

                builder = builder.add_source(File::from(file).format(format));
            } else {
                return Err(config_err(format!(
                    "Config file does not exist: {}",
                    file.display()
                )));
            }
        }

        // Add environment variables
        let prefix = self.env_prefix.unwrap_or_else(|| "MCP".to_string());
        builder = builder.add_source(
            Environment::with_prefix(&prefix)
                .separator("__")
                .try_parsing(true)
        );

        // Build configuration
        let config = builder.build()
            .map_err(|e| config_err(format!("Failed to build configuration: {}", e)))?;

        // Deserialize into our AppConfig
        let app_config: AppConfig = config.try_deserialize()
            .map_err(|e| config_err(format!("Failed to deserialize configuration: {}", e)))?;

        // Validate configuration
        app_config.validate()
            .map_err(|e| config_err(format!("Configuration validation failed: {}", e)))?;

        Ok(Arc::new(app_config))
    }
}

// Default value helper functions
fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_log_format() -> String { "plain".to_string() }
fn default_bind_address() -> String { "127.0.0.1".to_string() }
fn default_port() -> u16 { 8080 }
fn default_timeout() -> u64 { 30 }
fn default_max_request_size() -> usize { 10_485_760 } // 10 MB
fn default_keep_alive() -> u64 { 75 }
fn default_wasm_memory_limit() -> usize { 256 }
fn default_max_instances() -> usize { 10 }
fn default_app_name() -> String { "mcp-ectors".to_string() }
fn default_environment() -> String { "development".to_string() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validates() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(LogLevel::from_str("trace").unwrap(), LogLevel::Trace);
        assert_eq!(LogLevel::from_str("DEBUG").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("info").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("WARN").unwrap(), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("error").unwrap(), LogLevel::Error);
        assert!(LogLevel::from_str("invalid").is_err());
    }

    #[test]
    fn test_transport_config_validation() {
        // Valid config
        let config = TransportConfig::default();
        assert!(config.validate().is_ok());

        // Invalid: Only cert without key
        let mut config = TransportConfig::default();
        config.tls_cert = Some(PathBuf::from("/tmp/cert.pem"));
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_security_config_validation() {
        // Valid config
        let config = SecurityConfig::default();
        assert!(config.validate().is_ok());

        // Invalid: Auth enabled but no JWT secret or API keys
        let mut config = SecurityConfig::default();
        config.auth_enabled = true;
        assert!(config.validate().is_err());

        // Valid: Auth enabled with JWT secret
        let mut config = SecurityConfig::default();
        config.auth_enabled = true;
        config.jwt_secret = Some("this_is_a_very_long_secret_for_jwt_tokens_32+".to_string());
        assert!(config.validate().is_ok());

        // Invalid: JWT secret too short
        let mut config = SecurityConfig::default();
        config.auth_enabled = true;
        config.jwt_secret = Some("too_short".to_string());
        assert!(config.validate().is_err());
    }
}
