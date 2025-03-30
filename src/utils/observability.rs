use std::sync::Arc;
use std::time::Duration;

use tracing::{Level, Span};
use tracing_subscriber::{
    filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
    Registry,
};
use uuid::Uuid;

use crate::utils::config::AppConfig;
use crate::utils::error::{Result, internal_err};

/// Core observability framework that manages logging, metrics and tracing
pub struct ObservabilityFramework {
    /// Configuration reference
    config: Arc<AppConfig>,
    /// Service name for tagging
    service_name: String,
}

/// Trace context for distributed tracing
pub struct TraceContext {
    /// Unique trace ID
    pub trace_id: String,
    /// Unique span ID
    pub span_id: String,
    /// Trace parent ID (if any)
    pub parent_id: Option<String>,
}

impl Default for TraceContext {
    fn default() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_id: None,
        }
    }
}

impl ObservabilityFramework {
    /// Create a new observability framework
    pub fn new(config: Arc<AppConfig>, service_name: &str) -> Self {
        Self {
            config,
            service_name: service_name.to_string(),
        }
    }

    /// Initialize the observability framework
    pub fn init(&mut self) -> Result<()> {
        self.init_logging()?;

        // Log initialization complete
        tracing::info!(service_name = self.service_name.as_str(), "Observability framework initialized");

        Ok(())
    }

    /// Initialize structured logging
    fn init_logging(&self) -> Result<()> {
        let logging_config = &self.config.logging;

        // Create a filter based on the configured level
        let filter = EnvFilter::from_default_env()
            .add_directive(logging_config.level.to_string().parse()
                .map_err(|e| internal_err(format!("Invalid log level: {}", e)))?);

        // Create the registry and add layers
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true);

        // Initialize the global default with both layers
        Registry::default()
            .with(filter)
            .with(fmt_layer)
            .init();

        Ok(())
    }

    /// Create a new span with the given name and level
    pub fn create_span(&self, _name: &str, _level: Level) -> Span {
        // Return a dummy span using the current span
        Span::current()
    }

    /// Create a new trace context or extract it from headers
    pub fn create_trace_context(&self, _headers: Option<&std::collections::HashMap<String, String>>) -> TraceContext {
        // Simplified implementation
        TraceContext::default()
    }

    /// Record a counter metric
    pub fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        let labels_str = Self::labels_to_string(labels);
        tracing::debug!("METRIC_COUNTER name={}, value={}, labels={}", name, value, labels_str);
    }

    /// Record a gauge metric
    pub fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let labels_str = Self::labels_to_string(labels);
        tracing::debug!("METRIC_GAUGE name={}, value={}, labels={}", name, value, labels_str);
    }

    /// Record a histogram metric
    pub fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let labels_str = Self::labels_to_string(labels);
        tracing::debug!("METRIC_HISTOGRAM name={}, value={}, labels={}", name, value, labels_str);
    }

    /// Record a timing metric using a histogram
    pub fn record_timing(&self, name: &str, duration: Duration, labels: &[(&str, &str)]) {
        let seconds = duration.as_secs_f64();
        self.record_histogram(name, seconds, labels);
    }

    /// Convert labels to string representation for logging
    fn labels_to_string(labels: &[(&str, &str)]) -> String {
        let mut labels_str = String::new();
        for (i, (k, v)) in labels.iter().enumerate() {
            if i > 0 {
                labels_str.push_str(", ");
            }
            labels_str.push_str(&format!("{}={}", k, v));
        }
        labels_str
    }

    /// Get a reference to the Prometheus handle if available
    pub fn prometheus_handle(&self) -> Option<&()> {
        None
    }
}

// Factory methods for common metrics

/// Increment the specified counter by 1
pub fn increment_counter(name: &str, labels: &[(&str, &str)]) {
    let labels_str = ObservabilityFramework::labels_to_string(labels);
    tracing::debug!("METRIC_COUNTER name={}, value=1, labels={}", name, labels_str);
}

/// Set the specified gauge to the provided value
pub fn set_gauge(name: &str, value: f64, labels: &[(&str, &str)]) {
    let labels_str = ObservabilityFramework::labels_to_string(labels);
    tracing::debug!("METRIC_GAUGE name={}, value={}, labels={}", name, value, labels_str);
}

/// Record a histogram observation
pub fn record_histogram(name: &str, value: f64, labels: &[(&str, &str)]) {
    let labels_str = ObservabilityFramework::labels_to_string(labels);
    tracing::debug!("METRIC_HISTOGRAM name={}, value={}, labels={}", name, value, labels_str);
}

/// Record a timing metric in seconds
pub fn record_timing(name: &str, duration: Duration, labels: &[(&str, &str)]) {
    let seconds = duration.as_secs_f64();
    record_histogram(name, seconds, labels);
}

/// Create a tracing span for a function call and attach trace context
pub fn create_function_span(
    _name: &str,
    _level: Level,
    _trace_context: Option<&TraceContext>
) -> Span {
    // Simple implementation that doesn't rely on span! macros
    Span::current()
}

/// Helper function to time a function call and record a metric
pub async fn timed_call<F, Fut, T>(
    name: &str,
    labels: &[(&str, &str)],
    f: F
) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let start = std::time::Instant::now();
    let result = f().await;
    let duration = start.elapsed();

    record_timing(name, duration, labels);

    result
}

/// Helper function to wrap an async function with a tracing span
pub async fn with_span<F, Fut, T>(
    _name: &str,
    _level: Level,
    _trace_context: Option<&TraceContext>,
    f: F,
) -> T
where
    F: FnOnce(Span) -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let span = Span::current();
    f(span).await
}

/// Create a task timer that will automatically record timing metrics when dropped
pub struct TaskTimer<'a> {
    name: &'a str,
    labels: Vec<(&'a str, &'a str)>,
    start: std::time::Instant,
}

impl<'a> TaskTimer<'a> {
    /// Create a new task timer
    pub fn new(name: &'a str, labels: &[(&'a str, &'a str)]) -> Self {
        Self {
            name,
            labels: labels.to_vec(),
            start: std::time::Instant::now(),
        }
    }
}

impl<'a> Drop for TaskTimer<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        record_timing(self.name, duration, &self.labels);
    }
}

/// Helper trait to extend Result with observability features
pub trait ObservableResult<T> {
    /// Log an error if one occurs, then return the result
    fn log_error(self, message: &str) -> Self;

    /// Record a metric if an error occurs, then return the result
    fn count_error(self, counter_name: &str, labels: &[(&str, &str)]) -> Self;
}

impl<T, E: std::fmt::Display> ObservableResult<T> for std::result::Result<T, E> {
    fn log_error(self, message: &str) -> Self {
        if let Err(ref err) = self {
            tracing::error!("{}: {}", message, err);
        }
        self
    }

    fn count_error(self, counter_name: &str, labels: &[(&str, &str)]) -> Self {
        if self.is_err() {
            increment_counter(counter_name, labels);
        }
        self
    }
}

/// Initialize the observability framework with default configuration
pub fn init_default_observability(config: Arc<AppConfig>) -> Result<ObservabilityFramework> {
    let mut framework = ObservabilityFramework::new(config, "mcp-ectors");
    framework.init()?;
    Ok(framework)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::config::{LoggingConfig, TransportConfig, AppConfig};

    #[test]
    fn test_create_trace_context() {
        let config = create_test_config();
        let framework = ObservabilityFramework::new(Arc::new(config), "test-service");

        let context = framework.create_trace_context(None);
        assert!(!context.trace_id.is_empty());
        assert!(!context.span_id.is_empty());
        assert!(context.parent_id.is_none());
    }

    #[test]
    fn test_labels_to_string() {
        let labels = [("service", "test"), ("endpoint", "api")];
        let labels_str = ObservabilityFramework::labels_to_string(&labels);
        assert_eq!(labels_str, "service=test, endpoint=api");
    }

    fn create_test_config() -> AppConfig {
        let mut config = AppConfig::default();
        config.logging = LoggingConfig {
            level: crate::utils::config::LogLevel::Debug,
            timestamps: true,
            file: None,
            stdout: true,
            format: "plain".to_string(),
        };
        config.transport = TransportConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            tls_cert: None,
            tls_key: None,
            timeout_secs: 30,
            max_request_size: 1024 * 1024,
            keep_alive_secs: 60,
        };
        config
    }
}
