use std::sync::Arc;
use std::time::Duration;

use tracing::{info, warn, error, debug, Level};
use tokio::time::sleep;

use crate::utils::config::AppConfig;
use crate::utils::observability::{
    ObservabilityFramework, TaskTimer, increment_counter,
    set_gauge, record_histogram, record_timing, with_span,
    timed_call, ObservableResult,
};
use crate::utils::error::Result;

/// Example demonstrating the observability framework
pub async fn observability_example() -> Result<()> {
    println!("\n=== Observability Framework Example ===\n");

    // 1. Create a configuration for the example
    let config = Arc::new(AppConfig::default());

    // 2. Initialize the observability framework
    println!("Initializing observability framework...");
    let mut framework = ObservabilityFramework::new(config, "observability-example");
    framework.init()?;

    // 3. Log messages with different levels
    println!("\nLogging examples:");
    print_log_examples();

    // 4. Record metrics
    println!("\nRecording metrics:");
    record_metric_examples();

    // 5. Use tracing spans
    println!("\nTracing examples:");
    await_traced_functions().await;

    // 6. Use task timing
    println!("\nTask timing example:");
    time_operations();

    // 7. Use observable results
    println!("\nObservable results example:");
    demonstrate_observable_results()?;

    println!("\n=== End of Observability Example ===\n");

    Ok(())
}

/// Demonstrates tracing macros for structured logging
fn print_log_examples() {
    debug!(value = 42, component = "example", "This is a debug message with structured data");
    info!(request_id = "abc-123", "This is an info message with context");
    warn!(endpoint = "/api/example", "This is a warning message with details");
    error!(error_code = 404, reason = "Not found", "This is an error message");

    // Log with nested data
    info!(
        user.id = "user-123",
        user.role = "admin",
        "User information logged"
    );

    // Log timing information
    let duration = Duration::from_millis(123);
    info!(duration_ms = duration.as_millis(), "Operation completed");
}

/// Demonstrates recording metrics
fn record_metric_examples() {
    // Record counters
    increment_counter("api_requests_total", &[("method", "GET"), ("endpoint", "/users")]);
    increment_counter("api_requests_total", &[("method", "POST"), ("endpoint", "/users")]);

    // Record gauges
    set_gauge("active_connections", 42.0, &[("service", "api")]);
    set_gauge("queue_depth", 7.0, &[("queue", "background_jobs")]);

    // Record histograms
    record_histogram("response_size_bytes", 2048.0, &[("endpoint", "/users")]);

    // Record timing
    let operation_duration = Duration::from_millis(123);
    record_timing("operation_duration_seconds", operation_duration, &[("operation", "database_query")]);
}

/// Demonstrates async functions with tracing
async fn await_traced_functions() {
    // Call a traced function and await its result
    let result = with_span(
        "parent_operation",
        Level::INFO,
        None,
        |_span| async move {
            // Perform some work in the span
            info!("Inside parent span");

    // Call a nested operation
    let nested_result = with_span(
        "nested_operation",
        Level::DEBUG,
        None,
        |_span| async {
            // Simulate some async work
            sleep(Duration::from_millis(10)).await;
            debug!("Inside nested span");
            "nested result"
        }
    ).await;

            // Return both results
            (nested_result, "parent result")
        }
    ).await;

    println!("Results from traced functions: {:?}", result);

    // Time an async operation
    let timed_result = timed_call(
        "timed_operation_duration_seconds",
        &[("operation", "async_operation")],
        || async {
            // Simulate some async work
            sleep(Duration::from_millis(25)).await;
            "timed result"
        }
    ).await;

    println!("Result from timed call: {}", timed_result);
}

/// Demonstrates using TaskTimer for timing blocks of code
fn time_operations() {
    // Create a task timer that records metrics on drop
    {
        let _timer = TaskTimer::new("block_operation_duration_seconds", &[("block", "first")]);

        // Simulate some work
        std::thread::sleep(Duration::from_millis(15));

        info!("Executed first timed block");
    } // Timer is dropped here, recording the duration

    // Another timed block
    {
        let _timer = TaskTimer::new("block_operation_duration_seconds", &[("block", "second")]);

        // Simulate some work
        std::thread::sleep(Duration::from_millis(35));

        info!("Executed second timed block");
    } // Timer is dropped here, recording the duration
}

/// Demonstrates using ObservableResult to handle and log errors
fn demonstrate_observable_results() -> Result<()> {
    // A function that might fail
    fn might_fail(should_fail: bool) -> std::result::Result<String, String> {
        if should_fail {
            Err("Operation failed".to_string())
        } else {
            Ok("Operation succeeded".to_string())
        }
    }

    // Log success case
    let success = might_fail(false)
        .log_error("Failed to execute operation")
        .count_error("operation_errors_total", &[("operation", "example")]);

    println!("Success result: {:?}", success);

    // Log failure case (will log the error and increment the counter)
    let failure = might_fail(true)
        .log_error("Failed to execute operation")
        .count_error("operation_errors_total", &[("operation", "example")]);

    println!("Failure result: {:?}", failure);

    Ok(())
}
