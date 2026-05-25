pub mod swarm_controller;
pub mod workspace_preset;
pub mod input_router;
pub mod message_bus;

pub use swarm_controller::{SwarmController, SwarmConfig, AgentId, SwarmEvent};
pub use workspace_preset::{WorkspacePreset, PanelConfig, PanelType, WorkspacePresetManager};
pub use input_router::{InputRouter, InputClassification, RoutingDecision};
pub use message_bus::{MessageBus, AgentMessage, MessageEnvelope, TaskStatus};
