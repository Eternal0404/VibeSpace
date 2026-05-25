use crate::message_bus::{MessageBus, MessageEnvelope, AgentMessage, MessageType, TaskStatus, MessageFilter};
use crate::workspace_preset::WorkspacePreset;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub model: String,
    pub system_prompt: Option<String>,
    pub max_concurrent_tasks: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "vibe-agent".to_string(),
            model: "claude-3-5-sonnet".to_string(),
            system_prompt: None,
            max_concurrent_tasks: 3,
        }
    }
}

pub struct Agent {
    pub id: AgentId,
    pub config: AgentConfig,
    pub status: AgentStatus,
    pub current_tasks: Vec<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Working,
    Waiting,
    Terminated,
}

pub struct SwarmController {
    agents: Arc<RwLock<HashMap<AgentId, Agent>>>,
    message_bus: MessageBus,
    config: SwarmConfig,
    event_sender: Option<mpsc::UnboundedSender<SwarmEvent>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwarmConfig {
    pub max_agents: usize,
    pub default_timeout: Duration,
    pub heartbeat_interval: Duration,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            max_agents: 10,
            default_timeout: Duration::from_secs(300),
            heartbeat_interval: Duration::from_secs(30),
        }
    }
}

impl Default for SwarmController {
    fn default() -> Self {
        Self::new(SwarmConfig::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SwarmEvent {
    AgentJoined { agent_id: AgentId },
    AgentLeft { agent_id: AgentId },
    TaskAssigned { agent_id: AgentId, task_id: Uuid },
    TaskCompleted { agent_id: AgentId, task_id: Uuid },
    TaskFailed { agent_id: AgentId, task_id: Uuid, error: String },
    MessageSent { from: AgentId, to: Option<AgentId>, message_id: Uuid },
    SwarmStarted { agent_count: usize },
    SwarmStopped,
}

impl SwarmController {
    pub fn new(config: SwarmConfig) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            message_bus: MessageBus::default(),
            config,
            event_sender: None,
        }
    }

    pub fn with_event_channel(mut self, sender: mpsc::UnboundedSender<SwarmEvent>) -> Self {
        self.event_sender = Some(sender);
        self
    }

    pub fn message_bus(&self) -> &MessageBus {
        &self.message_bus
    }

    pub fn spawn_agent(&self, config: AgentConfig) -> Result<AgentId, SwarmError> {
        let mut agents = self.agents.write();

        if agents.len() >= self.config.max_agents {
            return Err(SwarmError::MaxAgentsReached(self.config.max_agents));
        }

        let id = AgentId::new();
        let agent = Agent {
            id,
            config,
            status: AgentStatus::Idle,
            current_tasks: Vec::new(),
        };

        agents.insert(id, agent);
        self.emit_event(SwarmEvent::AgentJoined { agent_id: id });

        Ok(id)
    }

    pub fn spawn_agents(&self, count: usize, base_config: AgentConfig) -> Result<Vec<AgentId>, SwarmError> {
        let mut ids = Vec::with_capacity(count);

        for i in 0..count {
            let mut config = base_config.clone();
            config.name = format!("{}-{}", base_config.name, i + 1);

            match self.spawn_agent(config) {
                Ok(id) => ids.push(id),
                Err(e) => {
                    for id in &ids {
                        let _ = self.terminate_agent(*id);
                    }
                    return Err(e);
                }
            }
        }

        self.emit_event(SwarmEvent::SwarmStarted { agent_count: ids.len() });
        Ok(ids)
    }

    pub fn terminate_agent(&self, agent_id: AgentId) -> Result<(), SwarmError> {
        let mut agents = self.agents.write();

        let agent = agents
            .get(&agent_id)
            .ok_or(SwarmError::AgentNotFound(agent_id))?;

        if !agent.current_tasks.is_empty() {
            return Err(SwarmError::AgentHasActiveTasks(agent_id));
        }

        agents.remove(&agent_id);
        self.emit_event(SwarmEvent::AgentLeft { agent_id });

        Ok(())
    }

    pub fn get_agent(&self, agent_id: &AgentId) -> Option<Agent> {
        self.agents.read().get(agent_id).cloned()
    }

    pub fn get_all_agents(&self) -> Vec<Agent> {
        self.agents.read().values().cloned().collect()
    }

    pub fn assign_task(&self, agent_id: AgentId, task: Task) -> Result<Uuid, SwarmError> {
        let mut agents = self.agents.write();

        let agent = agents
            .get_mut(&agent_id)
            .ok_or(SwarmError::AgentNotFound(agent_id))?;

        if agent.status == AgentStatus::Terminated {
            return Err(SwarmError::AgentTerminated(agent_id));
        }

        if agent.current_tasks.len() >= agent.config.max_concurrent_tasks {
            return Err(SwarmError::AgentTaskQueueFull(agent_id));
        }

        let task_id = task.id;
        agent.current_tasks.push(task_id);
        agent.status = AgentStatus::Working;

        self.emit_event(SwarmEvent::TaskAssigned { agent_id, task_id });

        Ok(task_id)
    }

    pub fn complete_task(&self, agent_id: AgentId, task_id: Uuid) -> Result<(), SwarmError> {
        let mut agents = self.agents.write();

        let agent = agents
            .get_mut(&agent_id)
            .ok_or(SwarmError::AgentNotFound(agent_id))?;

        agent.current_tasks.retain(|&id| id != task_id);

        if agent.current_tasks.is_empty() {
            agent.status = AgentStatus::Idle;
        }

        self.emit_event(SwarmEvent::TaskCompleted { agent_id, task_id });

        Ok(())
    }

    pub fn fail_task(&self, agent_id: AgentId, task_id: Uuid, error: String) -> Result<(), SwarmError> {
        let mut agents = self.agents.write();

        let agent = agents
            .get_mut(&agent_id)
            .ok_or(SwarmError::AgentNotFound(agent_id))?;

        agent.current_tasks.retain(|&id| id != task_id);

        if agent.current_tasks.is_empty() {
            agent.status = AgentStatus::Idle;
        }

        self.emit_event(SwarmEvent::TaskFailed { agent_id, task_id, error });

        Ok(())
    }

    pub fn send_message(
        &self,
        from: AgentId,
        to: Option<AgentId>,
        payload: serde_json::Value,
        message_type: MessageType,
    ) -> Result<Uuid, SwarmError> {
        let envelope = if let Some(target) = to {
            MessageEnvelope::new(from, Some(target), payload)
        } else {
            MessageEnvelope::broadcast(from, payload)
        };

        let message_id = envelope.id;
        self.message_bus.send_direct(envelope, message_type);

        self.emit_event(SwarmEvent::MessageSent { from, to, message_id });

        Ok(message_id)
    }

    pub fn delegate_task_to_agent(
        &self,
        from_agent: AgentId,
        to_agent: AgentId,
        subtask: SubTask,
    ) -> Result<Uuid, SwarmError> {
        let agents = self.agents.read();

        if !agents.contains_key(&from_agent) {
            return Err(SwarmError::AgentNotFound(from_agent));
        }
        if !agents.contains_key(&to_agent) {
            return Err(SwarmError::AgentNotFound(to_agent));
        }

        drop(agents);

        let payload = serde_json::to_value(&subtask).unwrap_or(serde_json::json!({}));
        self.send_message(from_agent, Some(to_agent), payload, MessageType::TaskDelegation)
    }

    pub fn request_peer_review(
        &self,
        from_agent: AgentId,
        target_agent: AgentId,
        review_request: PeerReviewRequest,
    ) -> Result<Uuid, SwarmError> {
        let payload = serde_json::to_value(&review_request).unwrap_or(serde_json::json!({}));
        self.send_message(from_agent, Some(target_agent), payload, MessageType::PeerReview)
    }

    pub fn share_context(
        &self,
        from_agent: AgentId,
        context: SharedContext,
    ) -> Result<Uuid, SwarmError> {
        let agents = self.agents.read();
        if !agents.contains_key(&from_agent) {
            return Err(SwarmError::AgentNotFound(from_agent));
        }
        drop(agents);

        let payload = serde_json::to_value(&context).unwrap_or(serde_json::json!({}));
        self.send_message(from_agent, None, payload, MessageType::ContextResponse)
    }

    pub fn wait_for_completion(&self, task_ids: &[Uuid], timeout: Duration) -> Vec<TaskResult> {
        let start = std::time::Instant::now();
        let mut results = Vec::new();

        while results.len() < task_ids.len() && start.elapsed() < timeout {
            let history = self.message_bus.get_history(None);
            for task_id in task_ids {
                if !results.iter().any(|r: &TaskResult| r.task_id == *task_id) {
                    if let Some(msg) = history.iter().find(|m| {
                        m.envelope.task_status.is_terminal()
                            && m.message_type == MessageType::ProgressUpdate
                    }) {
                        results.push(TaskResult {
                            task_id: *task_id,
                            status: msg.envelope.task_status,
                            result: msg.envelope.payload.clone(),
                        });
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        results
    }

    pub fn shutdown(&self) {
        let mut agents = self.agents.write();
        for (id, agent) in agents.drain() {
            if !agent.current_tasks.is_empty() {
                self.emit_event(SwarmEvent::TaskFailed {
                    agent_id: id,
                    task_id: agent.current_tasks[0],
                    error: "Swarm shutdown".to_string(),
                });
            }
        }
        self.emit_event(SwarmEvent::SwarmStopped);
    }

    fn emit_event(&self, event: SwarmEvent) {
        if let Some(ref sender) = self.event_sender {
            let _ = sender.send(event);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
    pub payload: serde_json::Value,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

impl Task {
    pub fn new(description: String, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            description,
            payload,
            deadline: None,
        }
    }

    pub fn with_deadline(mut self, deadline: chrono::DateTime<chrono::Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub parent_task_id: Uuid,
    pub description: String,
    pub payload: serde_json::Value,
    pub priority: TaskPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerReviewRequest {
    pub task_id: Uuid,
    pub changes_summary: String,
    pub files_modified: Vec<String>,
    pub requester_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedContext {
    pub context_type: ContextType,
    pub data: serde_json::Value,
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextType {
    FileContents,
    DirectoryStructure,
    GitDiff,
    TestResults,
    BuildOutput,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub result: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum SwarmError {
    #[error("Maximum number of agents ({0}) reached")]
    MaxAgentsReached(usize),

    #[error("Agent not found: {0:?}")]
    AgentNotFound(AgentId),

    #[error("Agent {0:?} has active tasks and cannot be terminated")]
    AgentHasActiveTasks(AgentId),

    #[error("Agent {0:?} is terminated and cannot accept tasks")]
    AgentTerminated(AgentId),

    #[error("Agent {0:?} task queue is full")]
    AgentTaskQueueFull(AgentId),

    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("Message bus error: {0}")]
    MessageBusError(String),
}

impl Clone for SwarmController {
    fn clone(&self) -> Self {
        Self {
            agents: Arc::clone(&self.agents),
            message_bus: self.message_bus.clone(),
            config: self.config,
            event_sender: self.event_sender.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_single_agent() {
        let controller = SwarmController::default();
        let config = AgentConfig::default();

        let result = controller.spawn_agent(config);
        assert!(result.is_ok());

        let agent_id = result.unwrap();
        let agent = controller.get_agent(&agent_id);
        assert!(agent.is_some());
        assert_eq!(agent.unwrap().status, AgentStatus::Idle);
    }

    #[test]
    fn test_spawn_multiple_agents() {
        let controller = SwarmController::default();
        let config = AgentConfig::default();

        let result = controller.spawn_agents(3, config);
        assert!(result.is_ok());

        let ids = result.unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_max_agents_limit() {
        let mut config = SwarmConfig::default();
        config.max_agents = 2;

        let controller = SwarmController::new(config);
        let base_config = AgentConfig::default();

        assert!(controller.spawn_agents(2, base_config.clone()).is_ok());
        assert!(controller.spawn_agents(1, base_config).is_err());
    }

    #[test]
    fn test_task_assignment() {
        let controller = SwarmController::default();
        let agent_id = controller.spawn_agent(AgentConfig::default()).unwrap();

        let task = Task::new("Test task".to_string(), serde_json::json!({}));
        let result = controller.assign_task(agent_id, task);

        assert!(result.is_ok());
        let agent = controller.get_agent(&agent_id).unwrap();
        assert_eq!(agent.status, AgentStatus::Working);
    }

    #[test]
    fn test_task_completion() {
        let controller = SwarmController::default();
        let agent_id = controller.spawn_agent(AgentConfig::default()).unwrap();

        let task = Task::new("Test task".to_string(), serde_json::json!({}));
        let task_id = controller.assign_task(agent_id, task).unwrap();

        let result = controller.complete_task(agent_id, task_id);
        assert!(result.is_ok());

        let agent = controller.get_agent(&agent_id).unwrap();
        assert_eq!(agent.status, AgentStatus::Idle);
    }
}
