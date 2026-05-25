use crate::swarm_controller::{AgentId, SwarmController};
use crate::workspace_preset::WorkspacePresetManager;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputClassification {
    ShellCommand,
    NaturalLanguage,
    SlashCommand,
    WorkspaceCommand,
    SwarmCommand,
}

impl InputClassification {
    pub fn is_agent_routed(&self) -> bool {
        matches!(
            self,
            InputClassification::NaturalLanguage
                | InputClassification::SlashCommand
                | InputClassification::SwarmCommand
                | InputClassification::WorkspaceCommand
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub classification: InputClassification,
    pub execute_as_shell: bool,
    pub route_to_agent: Option<AgentId>,
    pub agent_message: Option<AgentMessagePayload>,
    pub workspace_command: Option<WorkspaceCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessagePayload {
    pub message: String,
    pub context_files: Vec<String>,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceCommand {
    LoadPreset(String),
    SaveCurrentAsPreset(String),
    ListPresets,
    DeletePreset(String),
    ResetLayout,
}

pub struct InputRouter {
    swarm_controller: Arc<SwarmController>,
    preset_manager: Arc<RwLock<WorkspacePresetManager>>,
    default_agent_id: Option<AgentId>,
    command_prefixes: CommandPrefixes,
}

#[derive(Debug, Clone)]
pub struct CommandPrefixes {
    pub agent_prefixes: Vec<String>,
    pub swarm_prefixes: Vec<String>,
    pub workspace_prefixes: Vec<String>,
    pub slash_commands: Vec<String>,
}

impl Default for CommandPrefixes {
    fn default() -> Self {
        Self {
            agent_prefixes: vec![
                "@agent".to_string(),
                "@ai".to_string(),
                "ai:".to_string(),
                "agent:".to_string(),
            ],
            swarm_prefixes: vec![
                "@swarm".to_string(),
                "swarm:".to_string(),
                "//swarm".to_string(),
                "/swarm".to_string(),
            ],
            workspace_prefixes: vec![
                "/workspace".to_string(),
                "/preset".to_string(),
                "/layout".to_string(),
            ],
            slash_commands: vec![
                "/quit".to_string(),
                "/exit".to_string(),
                "/clear".to_string(),
                "/help".to_string(),
                "/save".to_string(),
                "/load".to_string(),
            ],
        }
    }
}

impl InputRouter {
    pub fn new(
        swarm_controller: Arc<SwarmController>,
        preset_manager: Arc<RwLock<WorkspacePresetManager>>,
    ) -> Self {
        Self {
            swarm_controller,
            preset_manager,
            default_agent_id: None,
            command_prefixes: CommandPrefixes::default(),
        }
    }

    pub fn with_default_agent(mut self, agent_id: AgentId) -> Self {
        self.default_agent_id = Some(agent_id);
        self
    }

    pub fn classify(&self, input: &str) -> InputClassification {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return InputClassification::ShellCommand;
        }

        for prefix in &self.command_prefixes.swarm_prefixes {
            if trimmed.starts_with(prefix) {
                return InputClassification::SwarmCommand;
            }
        }

        for prefix in &self.command_prefixes.workspace_prefixes {
            if trimmed.starts_with(prefix) {
                return InputClassification::WorkspaceCommand;
            }
        }

        for prefix in &self.command_prefixes.agent_prefixes {
            if trimmed.starts_with(prefix) {
                return InputClassification::NaturalLanguage;
            }
        }

        if trimmed.starts_with('/') {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if let Some(cmd) = parts.first() {
                if self
                    .command_prefixes
                    .slash_commands
                    .iter()
                    .any(|s| s.as_str() == *cmd)
                {
                    return InputClassification::SlashCommand;
                }
            }
        }

        if self.looks_like_shell_command(trimmed) {
            InputClassification::ShellCommand
        } else {
            InputClassification::NaturalLanguage
        }
    }

    fn looks_like_shell_command(&self, input: &str) -> bool {
        let shell_indicators = [
            "cd ",
            "ls",
            "mkdir",
            "rm ",
            "cp ",
            "mv ",
            "cat ",
            "grep ",
            "find ",
            "chmod",
            "chown",
            "sudo",
            "docker ",
            "git ",
            "npm ",
            "yarn ",
            "cargo ",
            "make ",
            "cmake",
            "pip ",
            "python",
            "node ",
            "ruby",
            "go ",
            "rustc",
            "gcc",
            "g++",
            "curl ",
            "wget ",
            "ssh ",
            "scp ",
            "rsync ",
            "tar ",
            "zip ",
            "unzip",
            "awk ",
            "sed ",
            "cut ",
            "sort ",
            "uniq ",
            "wc ",
            "head ",
            "tail ",
            "less ",
            "more ",
            "diff ",
            "patch ",
            "echo ",
            "printf ",
            "export ",
            "source ",
            "alias ",
            "unalias ",
            "type ",
            "which ",
            "whereis ",
            "ps ",
            "kill ",
            "killall ",
            "pkill ",
            "top ",
            "htop ",
            "df ",
            "du ",
            "free ",
            "uname ",
            "hostname ",
            "ifconfig ",
            "ip ",
            "netstat ",
            "ss ",
            "ping ",
            "traceroute ",
            "nslookup ",
            "dig ",
            "iptables ",
            "ufw ",
        ];

        for indicator in shell_indicators {
            if input.starts_with(indicator) {
                return true;
            }
        }

        if input.contains('|') || input.contains('>') || input.contains('<') || input.contains('&')
        {
            return true;
        }

        if input.contains('$') && (input.contains("$$") || input.starts_with('$')) {
            return true;
        }

        false
    }

    pub fn route(&self, input: &str) -> RoutingDecision {
        let classification = self.classify(input);

        match classification {
            InputClassification::ShellCommand => RoutingDecision {
                classification,
                execute_as_shell: true,
                route_to_agent: None,
                agent_message: None,
                workspace_command: None,
            },

            InputClassification::NaturalLanguage => {
                let message = self.strip_prefixes(input);
                let agent_message = AgentMessagePayload {
                    message,
                    context_files: Vec::new(),
                    flags: Vec::new(),
                };

                RoutingDecision {
                    classification,
                    execute_as_shell: false,
                    route_to_agent: self.default_agent_id,
                    agent_message: Some(agent_message),
                    workspace_command: None,
                }
            }

            InputClassification::SwarmCommand => self.handle_swarm_command(input),

            InputClassification::WorkspaceCommand => self.handle_workspace_command(input),

            InputClassification::SlashCommand => self.handle_slash_command(input),
        }
    }

    fn strip_prefixes(&self, input: &str) -> String {
        let mut result = input.trim().to_string();

        for prefix in &self.command_prefixes.agent_prefixes {
            if result.starts_with(prefix) {
                result = result
                    .strip_prefix(prefix)
                    .unwrap_or(&result)
                    .trim()
                    .to_string();
                break;
            }
        }

        result
    }

    fn handle_swarm_command(&self, input: &str) -> RoutingDecision {
        let trimmed = input.trim();

        let command_parts: Vec<&str> = if trimmed.starts_with('/') || trimmed.starts_with("//") {
            trimmed[1..].split_whitespace().collect()
        } else {
            let mut result_vec = Vec::new();
            let mut found = false;
            for prefix in &self.command_prefixes.swarm_prefixes {
                if trimmed.starts_with(prefix) {
                    let stripped = trimmed.strip_prefix(prefix).unwrap_or(trimmed).trim();
                    result_vec = stripped.split_whitespace().collect();
                    found = true;
                    break;
                }
            }
            if !found {
                result_vec = trimmed.split_whitespace().collect();
            }
            result_vec
        };

        if command_parts.is_empty() {
            return RoutingDecision {
                classification: InputClassification::SwarmCommand,
                execute_as_shell: false,
                route_to_agent: self.default_agent_id,
                agent_message: Some(AgentMessagePayload {
                    message: input.to_string(),
                    context_files: Vec::new(),
                    flags: Vec::new(),
                }),
                workspace_command: None,
            };
        }

        let subcommand = command_parts[0].to_lowercase();
        let args: Vec<&str> = command_parts[1..].to_vec();
        let args_str = args.join(" ");

        match subcommand.as_str() {
            "spawn" => {
                let count = args
                    .first()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1);
                RoutingDecision {
                    classification: InputClassification::SwarmCommand,
                    execute_as_shell: false,
                    route_to_agent: None,
                    agent_message: Some(AgentMessagePayload {
                        message: format!("[SYSTEM] Spawn {} agents", count),
                        context_files: Vec::new(),
                        flags: vec!["spawn".to_string(), count.to_string()],
                    }),
                    workspace_command: None,
                }
            }

            "status" => RoutingDecision {
                classification: InputClassification::SwarmCommand,
                execute_as_shell: false,
                route_to_agent: None,
                agent_message: Some(AgentMessagePayload {
                    message: "[SYSTEM] Request swarm status".to_string(),
                    context_files: Vec::new(),
                    flags: vec!["status".to_string()],
                }),
                workspace_command: None,
            },

            "delegate" => RoutingDecision {
                classification: InputClassification::SwarmCommand,
                execute_as_shell: false,
                route_to_agent: self.default_agent_id,
                agent_message: Some(AgentMessagePayload {
                    message: format!("[DELEGATE] {}", args_str),
                    context_files: Vec::new(),
                    flags: vec!["delegate".to_string()],
                }),
                workspace_command: None,
            },

            "broadcast" => RoutingDecision {
                classification: InputClassification::SwarmCommand,
                execute_as_shell: false,
                route_to_agent: None,
                agent_message: Some(AgentMessagePayload {
                    message: format!("[BROADCAST] {}", args_str),
                    context_files: Vec::new(),
                    flags: vec!["broadcast".to_string()],
                }),
                workspace_command: None,
            },

            "terminate" => RoutingDecision {
                classification: InputClassification::SwarmCommand,
                execute_as_shell: false,
                route_to_agent: None,
                agent_message: Some(AgentMessagePayload {
                    message: format!("[SYSTEM] Terminate agents: {}", args_str),
                    context_files: Vec::new(),
                    flags: vec!["terminate".to_string(), args_str],
                }),
                workspace_command: None,
            },

            _ => RoutingDecision {
                classification: InputClassification::SwarmCommand,
                execute_as_shell: false,
                route_to_agent: self.default_agent_id,
                agent_message: Some(AgentMessagePayload {
                    message: input.to_string(),
                    context_files: Vec::new(),
                    flags: Vec::new(),
                }),
                workspace_command: None,
            },
        }
    }

    fn handle_workspace_command(&self, input: &str) -> RoutingDecision {
        let trimmed = input.trim();

        let (command, arg) = if let Some(stripped) = trimmed.strip_prefix("/workspace") {
            let parts: Vec<&str> = stripped.trim().splitn(2, ' ').collect();
            (parts[0].trim(), parts.get(1).map(|s| s.trim()))
        } else if let Some(stripped) = trimmed.strip_prefix("/preset") {
            let parts: Vec<&str> = stripped.trim().splitn(2, ' ').collect();
            (parts[0].trim(), parts.get(1).map(|s| s.trim()))
        } else if let Some(stripped) = trimmed.strip_prefix("/layout") {
            let parts: Vec<&str> = stripped.trim().splitn(2, ' ').collect();
            (parts[0].trim(), parts.get(1).map(|s| s.trim()))
        } else {
            ("", None)
        };

        let workspace_command = match command {
            "load" | "open" => arg.map(|name| WorkspaceCommand::LoadPreset(name.to_string())),
            "save" => arg.map(|name| WorkspaceCommand::SaveCurrentAsPreset(name.to_string())),
            "list" | "ls" => Some(WorkspaceCommand::ListPresets),
            "delete" | "rm" => arg.map(|name| WorkspaceCommand::DeletePreset(name.to_string())),
            "reset" | "default" => Some(WorkspaceCommand::ResetLayout),
            _ => None,
        };

        RoutingDecision {
            classification: InputClassification::WorkspaceCommand,
            execute_as_shell: false,
            route_to_agent: None,
            agent_message: None,
            workspace_command,
        }
    }

    fn handle_slash_command(&self, input: &str) -> RoutingDecision {
        let trimmed = input.trim();
        let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        let command = parts[0];
        let _arg = parts.get(1);

        match command {
            "/quit" | "/exit" => RoutingDecision {
                classification: InputClassification::SlashCommand,
                execute_as_shell: false,
                route_to_agent: None,
                agent_message: Some(AgentMessagePayload {
                    message: "[SYSTEM] Exit requested".to_string(),
                    context_files: Vec::new(),
                    flags: vec!["exit".to_string()],
                }),
                workspace_command: None,
            },

            "/clear" => RoutingDecision {
                classification: InputClassification::SlashCommand,
                execute_as_shell: false,
                route_to_agent: None,
                agent_message: Some(AgentMessagePayload {
                    message: "[SYSTEM] Clear screen".to_string(),
                    context_files: Vec::new(),
                    flags: vec!["clear".to_string()],
                }),
                workspace_command: None,
            },

            "/help" => RoutingDecision {
                classification: InputClassification::SlashCommand,
                execute_as_shell: false,
                route_to_agent: None,
                agent_message: Some(AgentMessagePayload {
                    message: "[SYSTEM] Show help".to_string(),
                    context_files: Vec::new(),
                    flags: vec!["help".to_string()],
                }),
                workspace_command: None,
            },

            _ => RoutingDecision {
                classification: InputClassification::SlashCommand,
                execute_as_shell: true,
                route_to_agent: None,
                agent_message: None,
                workspace_command: None,
            },
        }
    }

    pub fn register_agent_prefix(&mut self, prefix: String) {
        if !self.command_prefixes.agent_prefixes.contains(&prefix) {
            self.command_prefixes.agent_prefixes.push(prefix);
        }
    }

    pub fn register_swarm_prefix(&mut self, prefix: String) {
        if !self.command_prefixes.swarm_prefixes.contains(&prefix) {
            self.command_prefixes.swarm_prefixes.push(prefix);
        }
    }
}

impl Clone for InputRouter {
    fn clone(&self) -> Self {
        Self {
            swarm_controller: Arc::clone(&self.swarm_controller),
            preset_manager: Arc::clone(&self.preset_manager),
            default_agent_id: self.default_agent_id,
            command_prefixes: self.command_prefixes.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_router() -> InputRouter {
        let swarm_controller = Arc::new(SwarmController::default());
        let preset_manager = Arc::new(RwLock::new(WorkspacePresetManager::new()));
        InputRouter::new(swarm_controller, preset_manager)
    }

    #[test]
    fn test_shell_command_detection() {
        let router = create_test_router();

        assert_eq!(router.classify("ls -la"), InputClassification::ShellCommand);
        assert_eq!(
            router.classify("cd /home"),
            InputClassification::ShellCommand
        );
        assert_eq!(
            router.classify("git status"),
            InputClassification::ShellCommand
        );
        assert_eq!(
            router.classify("npm install"),
            InputClassification::ShellCommand
        );
    }

    #[test]
    fn test_natural_language_detection() {
        let router = create_test_router();

        assert_eq!(
            router.classify("What files are in this directory?"),
            InputClassification::NaturalLanguage
        );
        assert_eq!(
            router.classify("Explain this code"),
            InputClassification::NaturalLanguage
        );
        assert_eq!(
            router.classify("How do I fix this bug?"),
            InputClassification::NaturalLanguage
        );
    }

    #[test]
    fn test_agent_prefix_stripping() {
        let router = create_test_router();

        let decision = router.route("@agent explain this function");
        assert!(!decision.execute_as_shell);
        assert!(decision.agent_message.is_some());
        assert!(decision
            .agent_message
            .unwrap()
            .message
            .contains("explain this function"));
    }

    #[test]
    fn test_workspace_commands() {
        let router = create_test_router();

        let decision = router.route("/workspace load fullstack");
        assert_eq!(
            decision.classification,
            InputClassification::WorkspaceCommand
        );
        assert!(decision.workspace_command.is_some());

        let decision = router.route("/preset list");
        assert_eq!(
            decision.classification,
            InputClassification::WorkspaceCommand
        );
    }

    #[test]
    fn test_swarm_commands() {
        let router = create_test_router();

        let decision = router.route("/swarm spawn 3");
        assert_eq!(decision.classification, InputClassification::SwarmCommand);
        assert!(decision.agent_message.is_some());
    }
}
