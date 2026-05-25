use crate::swarm_controller::AgentId;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Error,
    Cancelled,
}

impl TaskStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskStatus::Complete | TaskStatus::Error | TaskStatus::Cancelled
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub id: Uuid,
    pub sender_id: AgentId,
    pub target_id: Option<AgentId>,
    pub payload: serde_json::Value,
    pub task_status: TaskStatus,
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<Uuid>,
}

impl MessageEnvelope {
    pub fn new(sender_id: AgentId, target_id: Option<AgentId>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender_id,
            target_id,
            payload,
            task_status: TaskStatus::Pending,
            timestamp: Utc::now(),
            reply_to: None,
        }
    }

    pub fn broadcast(sender_id: AgentId, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender_id,
            target_id: None,
            payload,
            task_status: TaskStatus::Pending,
            timestamp: Utc::now(),
            reply_to: None,
        }
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.task_status = status;
        self
    }

    pub fn with_reply_to(mut self, reply_to: Uuid) -> Self {
        self.reply_to = Some(reply_to);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub envelope: MessageEnvelope,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    TaskDelegation,
    ProgressUpdate,
    ResultSharing,
    PeerReview,
    ContextRequest,
    ContextResponse,
    Termination,
    Heartbeat,
}

pub type MessageHandler = Arc<dyn Fn(&AgentMessage) + Send + Sync>;

pub struct Subscription {
    pub agent_id: AgentId,
    pub filter: MessageFilter,
    pub handler: Arc<dyn Fn(&AgentMessage) + Send + Sync>,
}

#[derive(Debug, Clone, Default)]
pub struct MessageFilter {
    pub sender_id: Option<AgentId>,
    pub target_id: Option<AgentId>,
    pub message_types: Vec<MessageType>,
}

impl MessageFilter {
    pub fn matches(&self, message: &AgentMessage) -> bool {
        if let Some(ref sender) = self.sender_id {
            if &message.envelope.sender_id != sender {
                return false;
            }
        }
        if let Some(ref target) = self.target_id {
            if message.envelope.target_id.as_ref() != Some(target) {
                return false;
            }
        }
        if !self.message_types.is_empty() && !self.message_types.contains(&message.message_type) {
            return false;
        }
        true
    }
}

pub struct MessageBus {
    subscriptions: Arc<RwLock<HashMap<Uuid, Subscription>>>,
    message_history: Arc<RwLock<Vec<AgentMessage>>>,
    max_history: usize,
}

impl MessageBus {
    pub fn new(max_history: usize) -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(Vec::new())),
            max_history,
        }
    }

    pub fn subscribe(
        &self,
        agent_id: AgentId,
        filter: MessageFilter,
        handler: impl Fn(&AgentMessage) + Send + Sync + 'static,
    ) -> uuid::Uuid {
        let id = Uuid::new_v4();
        let subscription = Subscription {
            agent_id,
            filter,
            handler: Arc::new(handler),
        };
        self.subscriptions.write().insert(id, subscription);
        id
    }

    pub fn unsubscribe(&self, subscription_id: Uuid) {
        self.subscriptions.write().remove(&subscription_id);
    }

    pub fn publish(&self, message: AgentMessage) {
        self.add_to_history(&message);
        let subscriptions = self.subscriptions.read();
        for subscription in subscriptions.values() {
            if subscription.filter.matches(&message) {
                (subscription.handler)(&message);
            }
        }
    }

    pub fn send_direct(&self, envelope: MessageEnvelope, message_type: MessageType) {
        let message = AgentMessage {
            envelope,
            message_type,
        };
        self.publish(message);
    }

    pub fn broadcast(
        &self,
        sender_id: AgentId,
        payload: serde_json::Value,
        message_type: MessageType,
    ) {
        let envelope = MessageEnvelope::broadcast(sender_id, payload);
        let message = AgentMessage {
            envelope,
            message_type,
        };
        self.publish(message);
    }

    pub fn reply_to(
        &self,
        original: &MessageEnvelope,
        payload: serde_json::Value,
        status: TaskStatus,
        message_type: MessageType,
    ) {
        let envelope = MessageEnvelope::new(original.sender_id, Some(original.sender_id), payload)
            .with_status(status)
            .with_reply_to(original.id);

        let message = AgentMessage {
            envelope,
            message_type,
        };
        self.publish(message);
    }

    pub fn get_history(&self, limit: Option<usize>) -> Vec<AgentMessage> {
        let history = self.message_history.read();
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.clone(),
        }
    }

    pub fn get_messages_for_agent(
        &self,
        agent_id: &AgentId,
        limit: Option<usize>,
    ) -> Vec<AgentMessage> {
        let history = self.message_history.read();
        let messages: Vec<AgentMessage> = history
            .iter()
            .filter(|m| {
                m.envelope.sender_id == *agent_id
                    || m.envelope.target_id.as_ref() == Some(agent_id)
                    || m.envelope.target_id.is_none()
            })
            .cloned()
            .collect();

        match limit {
            Some(n) => messages.into_iter().rev().take(n).collect(),
            None => messages,
        }
    }

    fn add_to_history(&self, message: &AgentMessage) {
        let mut history = self.message_history.write();
        if history.len() >= self.max_history {
            history.remove(0);
        }
        history.push(message.clone());
    }

    pub fn clear_history(&self) {
        self.message_history.write().clear();
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl Clone for MessageBus {
    fn clone(&self) -> Self {
        Self {
            subscriptions: Arc::clone(&self.subscriptions),
            message_history: Arc::clone(&self.message_history),
            max_history: self.max_history,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_envelope_creation() {
        let agent_id = AgentId::new();
        let envelope = MessageEnvelope::new(agent_id, None, serde_json::json!({"test": true}));

        assert_eq!(envelope.sender_id, agent_id);
        assert!(envelope.target_id.is_none());
        assert!(!envelope.reply_to.is_some());
    }

    #[test]
    fn test_broadcast() {
        let agent_id = AgentId::new();
        let envelope =
            MessageEnvelope::broadcast(agent_id, serde_json::json!({"type": "announcement"}));

        assert_eq!(envelope.sender_id, agent_id);
        assert!(envelope.target_id.is_none());
    }

    #[test]
    fn test_message_filter_matches() {
        let agent1 = AgentId::new();
        let agent2 = AgentId::new();

        let filter = MessageFilter {
            sender_id: Some(agent1),
            target_id: None,
            message_types: vec![MessageType::TaskDelegation],
        };

        let message = AgentMessage {
            envelope: MessageEnvelope::new(agent1, Some(agent2), serde_json::json!({})),
            message_type: MessageType::TaskDelegation,
        };

        assert!(filter.matches(&message));

        let different_sender = AgentMessage {
            envelope: MessageEnvelope::new(agent2, Some(agent1), serde_json::json!({})),
            message_type: MessageType::TaskDelegation,
        };

        assert!(!filter.matches(&different_sender));
    }
}
