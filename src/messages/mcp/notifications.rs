use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::utils::error::{Result, jsonrpc_err};
use crate::messages::JSONRPC_VERSION;
use mcp_spec::protocol::JsonRpcNotification;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};

/// MCP specific notification methods
pub mod notification_methods {
    /// Resource change notification
    pub const RESOURCE_CHANGED: &str = "mcp/resource/changed";
    /// Tool change notification
    pub const TOOL_CHANGED: &str = "mcp/tool/changed";
    /// Prompt change notification
    pub const PROMPT_CHANGED: &str = "mcp/prompt/changed";
    /// Server status notification
    pub const SERVER_STATUS: &str = "mcp/server/status";
    /// Router status notification
    pub const ROUTER_STATUS: &str = "mcp/router/status";
    /// Subscription confirmation
    pub const SUBSCRIPTION_CONFIRMED: &str = "mcp/subscription/confirmed";
    /// Subscription expired
    pub const SUBSCRIPTION_EXPIRED: &str = "mcp/subscription/expired";
}

/// Notification severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationSeverity {
    /// Informational message
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
}

/// Represents an MCP subscription for a specific topic
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Subscription {
    /// Unique subscription ID
    pub subscription_id: String,
    /// Topic pattern to subscribe to
    pub topic: String,
    /// Client ID associated with this subscription
    pub client_id: String,
    /// When the subscription was created (ISO 8601 timestamp)
    pub created_at: String,
    /// Optional expiration time (ISO 8601 timestamp)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expires_at: Option<String>,
    /// Filter criteria in JSON
    #[serde(default)]
    pub filter: Option<Value>,
}

impl Subscription {
    /// Creates a new subscription
    pub fn new(subscription_id: String, topic: String, client_id: String, expires_in: Option<Duration>, filter: Option<Value>) -> Self {
        let now = SystemTime::now();
        let created_at = DateTime::<Utc>::from(now).to_rfc3339();

        // Calculate expiration time
        let expires_at = expires_in.map(|duration| {
            let expiry_time = now + duration;
            DateTime::<Utc>::from(expiry_time).to_rfc3339()
        });

        Self {
            subscription_id,
            topic,
            client_id,
            created_at,
            expires_at,
            filter,
        }
    }

    /// Checks if a subscription is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at_str) = &self.expires_at {
            // Parse the ISO 8601 expiration time and compare with current time
            if let Ok(expires_at) = DateTime::parse_from_rfc3339(expires_at_str) {
                let now = Utc::now().with_timezone(&expires_at.timezone());
                return expires_at < now;
            }
        }
        false
    }

    /// Extends the subscription by the given duration
    pub fn extend(&mut self, duration: Duration) {
        let now = SystemTime::now();
        let base_time = if let Some(expires_at_str) = &self.expires_at {
            if let Ok(expires_at) = DateTime::parse_from_rfc3339(expires_at_str) {
                let now_fixed = Utc::now().with_timezone(&expires_at.timezone());
                if expires_at > now_fixed {
                    // If not expired, extend from current expiration
                    SystemTime::now() + Duration::from_secs((expires_at.timestamp() - now_fixed.timestamp()) as u64)
                } else {
                    // If expired, extend from now
                    now
                }
            } else {
                now
            }
        } else {
            now
        };

        let new_expiry = base_time + duration;
        self.expires_at = Some(DateTime::<Utc>::from(new_expiry).to_rfc3339());
    }

    /// Converts the subscription to a notification confirming subscription
    pub fn to_confirmation_notification(&self) -> Result<JsonRpcNotification> {
        let expires_in_secs = if let Some(expires_at_str) = &self.expires_at {
            if let Ok(expires_at) = DateTime::parse_from_rfc3339(expires_at_str) {
                let now = Utc::now().with_timezone(&expires_at.timezone());
                if expires_at > now {
                    Some((expires_at.timestamp() - now.timestamp()) as u64)
                } else {
                    Some(0)
                }
            } else {
                None
            }
        } else {
            None
        };

        let params = json!({
            "subscription_id": self.subscription_id,
            "topic": self.topic,
            "expires_in_secs": expires_in_secs,
            "filter": self.filter,
        });

        Ok(JsonRpcNotification {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: notification_methods::SUBSCRIPTION_CONFIRMED.to_string(),
            params: Some(params),
        })
    }
}

/// Notification payload format for MCP notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPayload {
    /// Topic of the notification (e.g., "mcp/resource/changed")
    pub topic: String,
    /// Optional subscription ID if this is related to a subscription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    /// Timestamp of when the notification was generated (ISO 8601)
    pub timestamp: String,
    /// Notification severity level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<NotificationSeverity>,
    /// Message describing the notification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Data specific to the notification type
    pub data: Value,
}

impl NotificationPayload {
    /// Creates a new notification payload
    pub fn new(topic: &str, data: Value) -> Self {
        Self {
            topic: topic.to_string(),
            subscription_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            severity: None,
            message: None,
            data,
        }
    }

    /// Sets the subscription ID
    pub fn with_subscription_id(mut self, subscription_id: String) -> Self {
        self.subscription_id = Some(subscription_id);
        self
    }

    /// Sets the severity level
    pub fn with_severity(mut self, severity: NotificationSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Sets the message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Converts the payload to a JsonRpcNotification
    pub fn to_notification(&self) -> Result<JsonRpcNotification> {
        Ok(JsonRpcNotification {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: self.topic.clone(),
            params: Some(json!(self)),
        })
    }
}

/// Resource change notification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceChangeData {
    /// Router ID that owns the resource
    pub router_id: String,
    /// URI of the resource that changed
    pub uri: String,
    /// Type of change (created, updated, deleted)
    pub change_type: String,
}

/// Tool change notification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChangeData {
    /// Router ID that owns the tool
    pub router_id: String,
    /// Name of the tool that changed
    pub tool_name: String,
    /// Type of change (created, updated, deleted)
    pub change_type: String,
}

/// Prompt change notification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptChangeData {
    /// Router ID that owns the prompt
    pub router_id: String,
    /// Name of the prompt that changed
    pub prompt_name: String,
    /// Type of change (created, updated, deleted)
    pub change_type: String,
}

/// Server status notification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusData {
    /// Status message
    pub status: String,
    /// Optional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Server metrics
    pub metrics: HashMap<String, Value>,
}

/// Router status notification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterStatusData {
    /// Router ID
    pub router_id: String,
    /// Status (online, offline, degraded)
    pub status: String,
    /// Optional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Manages subscriptions for clients
#[derive(Clone)]
pub struct SubscriptionManager {
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
    /// Subscriptions indexed by client ID
    client_subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl SubscriptionManager {
    /// Creates a new subscription manager
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            client_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new subscription
    pub fn create_subscription(&self, topic: String, client_id: String, expires_in: Option<Duration>, filter: Option<Value>) -> Result<Subscription> {
        let subscription_id = uuid::Uuid::new_v4().to_string();
        let subscription = Subscription::new(subscription_id.clone(), topic, client_id.clone(), expires_in, filter);

        // Add the subscription
        {
            let mut subscriptions = self.subscriptions.write().map_err(|e| {
                jsonrpc_err(format!("Failed to acquire write lock for subscriptions: {}", e))
            })?;
            subscriptions.insert(subscription_id.clone(), subscription.clone());
        }

        // Add to client subscriptions index
        {
            let mut client_subs = self.client_subscriptions.write().map_err(|e| {
                jsonrpc_err(format!("Failed to acquire write lock for client subscriptions: {}", e))
            })?;

            client_subs.entry(client_id).or_insert_with(Vec::new).push(subscription_id);
        }

        Ok(subscription)
    }

    /// Gets a subscription by ID
    pub fn get_subscription(&self, subscription_id: &str) -> Result<Option<Subscription>> {
        let subscriptions = self.subscriptions.read().map_err(|e| {
            jsonrpc_err(format!("Failed to acquire read lock for subscriptions: {}", e))
        })?;

        Ok(subscriptions.get(subscription_id).cloned())
    }

    /// Removes a subscription
    pub fn remove_subscription(&self, subscription_id: &str) -> Result<Option<Subscription>> {
        let removed = {
            let mut subscriptions = self.subscriptions.write().map_err(|e| {
                jsonrpc_err(format!("Failed to acquire write lock for subscriptions: {}", e))
            })?;

            subscriptions.remove(subscription_id)
        };

        // If we removed a subscription, clean up the client index
        if let Some(subscription) = &removed {
            let mut client_subs = self.client_subscriptions.write().map_err(|e| {
                jsonrpc_err(format!("Failed to acquire write lock for client subscriptions: {}", e))
            })?;

            if let Some(subs) = client_subs.get_mut(&subscription.client_id) {
                subs.retain(|id| id != subscription_id);

                // Clean up empty client entries
                if subs.is_empty() {
                    client_subs.remove(&subscription.client_id);
                }
            }
        }

        Ok(removed)
    }

    /// Gets all subscriptions for a client
    pub fn get_client_subscriptions(&self, client_id: &str) -> Result<Vec<Subscription>> {
        let client_subs = self.client_subscriptions.read().map_err(|e| {
            jsonrpc_err(format!("Failed to acquire read lock for client subscriptions: {}", e))
        })?;

        let subscriptions = self.subscriptions.read().map_err(|e| {
            jsonrpc_err(format!("Failed to acquire read lock for subscriptions: {}", e))
        })?;

        let subscription_ids = match client_subs.get(client_id) {
            Some(ids) => ids,
            None => return Ok(Vec::new()),
        };

        let client_subscriptions = subscription_ids
            .iter()
            .filter_map(|id| subscriptions.get(id).cloned())
            .collect();

        Ok(client_subscriptions)
    }

    /// Removes all subscriptions for a client
    pub fn remove_client_subscriptions(&self, client_id: &str) -> Result<Vec<Subscription>> {
        let subscription_ids = {
            let mut client_subs = self.client_subscriptions.write().map_err(|e| {
                jsonrpc_err(format!("Failed to acquire write lock for client subscriptions: {}", e))
            })?;

            client_subs.remove(client_id).unwrap_or_default()
        };

        let mut removed_subscriptions = Vec::new();
        let mut subscriptions = self.subscriptions.write().map_err(|e| {
            jsonrpc_err(format!("Failed to acquire write lock for subscriptions: {}", e))
        })?;

        for id in subscription_ids {
            if let Some(subscription) = subscriptions.remove(&id) {
                removed_subscriptions.push(subscription);
            }
        }

        Ok(removed_subscriptions)
    }

    /// Gets all subscriptions matching a topic
    pub fn get_subscriptions_for_topic(&self, topic: &str) -> Result<Vec<Subscription>> {
        let subscriptions = self.subscriptions.read().map_err(|e| {
            jsonrpc_err(format!("Failed to acquire read lock for subscriptions: {}", e))
        })?;

        let matching_subscriptions = subscriptions
            .values()
            .filter(|sub| sub.topic == topic || topic.starts_with(&sub.topic))
            .filter(|sub| !sub.is_expired())
            .cloned()
            .collect();

        Ok(matching_subscriptions)
    }

    /// Cleans up expired subscriptions
    pub fn cleanup_expired(&self) -> Result<Vec<Subscription>> {
        let mut expired = Vec::new();

        let subscription_ids = {
            let subscriptions = self.subscriptions.read().map_err(|e| {
                jsonrpc_err(format!("Failed to acquire read lock for subscriptions: {}", e))
            })?;

            subscriptions
                .iter()
                .filter(|(_, sub)| sub.is_expired())
                .map(|(id, _)| id.clone())
                .collect::<Vec<String>>()
        };

        for id in subscription_ids {
            if let Some(sub) = self.remove_subscription(&id)? {
                expired.push(sub);
            }
        }

        Ok(expired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_subscription_expiration() {
        let subscription = Subscription::new(
            "test-id".to_string(),
            "test-topic".to_string(),
            "test-client".to_string(),
            Some(Duration::from_millis(100)),
            None,
        );

        assert!(!subscription.is_expired());
        sleep(Duration::from_millis(150));
        assert!(subscription.is_expired());
    }

    #[test]
    fn test_notification_payload() {
        let data = json!({
            "key": "value",
            "number": 123
        });

        let payload = NotificationPayload::new(notification_methods::RESOURCE_CHANGED, data.clone())
            .with_severity(NotificationSeverity::Info)
            .with_message("Resource has changed".to_string());

        assert_eq!(payload.topic, notification_methods::RESOURCE_CHANGED);
        assert_eq!(payload.severity, Some(NotificationSeverity::Info));
        assert_eq!(payload.message, Some("Resource has changed".to_string()));
        assert_eq!(payload.data, data);
    }

    #[test]
    fn test_subscription_manager() {
        let manager = SubscriptionManager::new();

        // Create subscription
        let sub = manager.create_subscription(
            "test/topic".to_string(),
            "client1".to_string(),
            Some(Duration::from_secs(60)),
            Some(json!({"filter": "value"}))
        ).unwrap();

        // Get subscription
        let retrieved = manager.get_subscription(&sub.subscription_id).unwrap().unwrap();
        assert_eq!(retrieved.topic, "test/topic");

        // Get client subscriptions
        let client_subs = manager.get_client_subscriptions("client1").unwrap();
        assert_eq!(client_subs.len(), 1);

        // Get subscriptions for topic
        let topic_subs = manager.get_subscriptions_for_topic("test/topic").unwrap();
        assert_eq!(topic_subs.len(), 1);

        // Remove subscription
        let removed = manager.remove_subscription(&sub.subscription_id).unwrap().unwrap();
        assert_eq!(removed.subscription_id, sub.subscription_id);

        // Verify it's gone
        let empty = manager.get_subscription(&sub.subscription_id).unwrap();
        assert!(empty.is_none());
    }
}
