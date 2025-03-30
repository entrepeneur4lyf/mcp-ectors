use crate::utils::{
    AppConfig, ConfigBuilder, LogLevel, Result, ResultExt,
    LoggingConfig, TransportConfig, RouterConfig, SecurityConfig
};

/// Example demonstrating the configuration system
pub fn config_example() -> Result<()> {
    println!("\n=== Configuration System Example ===\n");

    // 1. Load configuration using the builder
    println!("Loading configuration from config.toml...");
    let config = ConfigBuilder::new()
        .with_config_file("config.toml")
        .with_env_prefix("MCP")
        .build()
        .context("Failed to load configuration")?;

    // 2. Display loaded configuration
    println!("\nApplication Configuration:\n");
    println!("Name: {}", config.name);
    println!("Environment: {}", config.environment);

    println!("\nLogging Configuration:");
    println!("  Level: {}", config.logging.level);
    println!("  Stdout: {}", if config.logging.stdout { "Enabled" } else { "Disabled" });
    if let Some(file) = &config.logging.file {
        println!("  Log File: {}", file.display());
    } else {
        println!("  Log File: Not configured");
    }
    println!("  Format: {}", config.logging.format);

    println!("\nTransport Configuration:");
    println!("  Bind Address: {}:{}", config.transport.bind_address, config.transport.port);
    if let (Some(cert), Some(key)) = (&config.transport.tls_cert, &config.transport.tls_key) {
        println!("  TLS: Enabled");
        println!("    Certificate: {}", cert.display());
        println!("    Key: {}", key.display());
    } else {
        println!("  TLS: Disabled");
    }
    println!("  Timeout: {} seconds", config.transport.timeout_secs);
    println!("  Max Request Size: {} bytes", config.transport.max_request_size);
    println!("  Keep Alive: {} seconds", config.transport.keep_alive_secs);

    println!("\nRouter Configuration:");
    if let Some(dir) = &config.router.wasm_dir {
        println!("  WASM Directory: {}", dir.display());
        println!("  Watch Directory: {}", if config.router.watch_wasm_dir { "Yes" } else { "No" });
    } else {
        println!("  WASM Directory: Not configured");
    }
    println!("  Memory Limit: {} MB", config.router.wasm_memory_limit_mb);
    println!("  Max Instances: {}", config.router.max_instances);
    println!("  Preload Routers: {}", if config.router.preload_routers { "Yes" } else { "No" });

    println!("\nSecurity Configuration:");
    println!("  Authentication: {}", if config.security.auth_enabled { "Enabled" } else { "Disabled" });
    if let Some(secret) = &config.security.jwt_secret {
        println!("  JWT Secret: Configured ({} chars)", secret.len());
    } else {
        println!("  JWT Secret: Not configured");
    }
    println!("  API Keys: {}", if config.security.api_keys.is_empty() { "None" } else { "Configured" });
    println!("  Allowed Origins: {}", config.security.allowed_origins.join(", "));
    println!("  Enforce HTTPS: {}", if config.security.enforce_https { "Yes" } else { "No" });

    if !config.custom.is_empty() {
        println!("\nCustom Settings:");
        for (key, value) in &config.custom {
            println!("  {}: {}", key, value);
        }
    }

    // 3. Demonstrate configuration override with environment variables
    println!("\nEnvironment Variable Override Example:");
    println!("  You can override settings with environment variables:");
    println!("    export MCP__LOGGING__LEVEL=debug");
    println!("    export MCP__TRANSPORT__PORT=9090");
    println!("    export MCP__SECURITY__AUTH_ENABLED=true");

    // 4. Demonstrate programmatic configuration creation
    println!("\nProgrammatic Configuration Example:");
    let custom_config = AppConfig {
        name: "custom-instance".to_string(),
        environment: "development".to_string(),
        logging: LoggingConfig {
            level: LogLevel::Debug,
            timestamps: true,
            file: None,
            stdout: true,
            format: "json".to_string(),
        },
        transport: TransportConfig {
            bind_address: "0.0.0.0".to_string(),
            port: 9000,
            tls_cert: None,
            tls_key: None,
            timeout_secs: 60,
            max_request_size: 5 * 1024 * 1024, // 5 MB
            keep_alive_secs: 60,
        },
        router: RouterConfig::default(),
        security: SecurityConfig::default(),
        custom: std::collections::HashMap::new(),
    };

    println!("  Custom instance name: {}", custom_config.name);
    println!("  Custom bind address: {}:{}", custom_config.transport.bind_address, custom_config.transport.port);
    println!("  Custom logging level: {}", custom_config.logging.level);

    println!("\n=== End of Configuration Example ===\n");

    Ok(())
}
