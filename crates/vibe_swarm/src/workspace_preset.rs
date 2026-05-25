use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelType {
    Terminal,
    Editor,
    Preview,
    AgentHub,
    FileExplorer,
    Output,
    Problems,
    DebugConsole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Pwsh,
    Cmd,
    WindowsCmd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    pub panel_id: Uuid,
    pub panel_type: PanelType,
    pub working_directory: PathBuf,
    pub shell_type: Option<ShellType>,
    pub width_ratio: f32,
    pub height_ratio: f32,
    pub x_position: f32,
    pub y_position: f32,
    pub is_visible: bool,
    pub title: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PanelConfig {
    pub fn new(
        panel_type: PanelType,
        working_directory: PathBuf,
        width_ratio: f32,
        height_ratio: f32,
    ) -> Self {
        Self {
            panel_id: Uuid::new_v4(),
            panel_type,
            working_directory,
            shell_type: None,
            width_ratio,
            height_ratio,
            x_position: 0.0,
            y_position: 0.0,
            is_visible: true,
            title: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_shell(mut self, shell_type: ShellType) -> Self {
        self.shell_type = Some(shell_type);
        self
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.x_position = x;
        self.y_position = y;
        self
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePreset {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub panels: Vec<PanelConfig>,
    pub active_panel_id: Option<Uuid>,
    pub layout_type: LayoutType,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutType {
    Single,
    HorizontalSplit,
    VerticalSplit,
    Grid,
    Custom,
}

impl WorkspacePreset {
    pub fn new(name: String, layout_type: LayoutType) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            created_at: now,
            updated_at: now,
            panels: Vec::new(),
            active_panel_id: None,
            layout_type,
            metadata: HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_panel(mut self, panel: PanelConfig) -> Self {
        self.panels.push(panel);
        self
    }

    pub fn with_active_panel(mut self, panel_id: Uuid) -> Self {
        self.active_panel_id = Some(panel_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn add_terminal_panel(
        &mut self,
        working_directory: PathBuf,
        shell_type: Option<ShellType>,
        position: PanelPosition,
    ) -> &mut PanelConfig {
        let (width, height, x, y) = position.to_dimensions();
        let mut panel = PanelConfig::new(PanelType::Terminal, working_directory, width, height)
            .with_position(x, y)
            .with_shell(shell_type.unwrap_or(ShellType::Bash));

        if let Some(title) = position.title {
            panel = panel.with_title(title);
        }

        self.panels.push(panel);
        self.panels.last_mut().unwrap()
    }

    pub fn add_preview_panel(&mut self, url: String, position: PanelPosition) -> &mut PanelConfig {
        let (width, height, x, y) = position.to_dimensions();
        let mut panel = PanelConfig::new(PanelType::Preview, PathBuf::new(), width, height)
            .with_position(x, y)
            .with_metadata("url".to_string(), serde_json::json!(url));

        if let Some(title) = position.title {
            panel = panel.with_title(title);
        }

        self.panels.push(panel);
        self.panels.last_mut().unwrap()
    }

    pub fn add_agent_hub_panel(&mut self, position: PanelPosition) -> &mut PanelConfig {
        let (width, height, x, y) = position.to_dimensions();
        let panel = PanelConfig::new(PanelType::AgentHub, PathBuf::new(), width, height)
            .with_position(x, y)
            .with_title("Agent Hub".to_string());

        self.panels.push(panel);
        self.panels.last_mut().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct PanelPosition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub title: Option<String>,
}

impl PanelPosition {
    pub fn to_dimensions(&self) -> (f32, f32, f32, f32) {
        (self.width, self.height, self.x, self.y)
    }

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            title: None,
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn fullscreen() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            title: None,
        }
    }

    pub fn left_half() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.5,
            height: 1.0,
            title: None,
        }
    }

    pub fn right_half() -> Self {
        Self {
            x: 0.5,
            y: 0.0,
            width: 0.5,
            height: 1.0,
            title: None,
        }
    }

    pub fn top_half() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 0.5,
            title: None,
        }
    }

    pub fn bottom_half() -> Self {
        Self {
            x: 0.0,
            y: 0.5,
            width: 1.0,
            height: 0.5,
            title: None,
        }
    }

    pub fn quadrant(top_left_x: f32, top_left_y: f32) -> Self {
        Self {
            x: top_left_x * 0.5,
            y: top_left_y * 0.5,
            width: 0.5,
            height: 0.5,
            title: None,
        }
    }
}

pub struct WorkspacePresetManager {
    presets: Arc<RwLock<HashMap<Uuid, WorkspacePreset>>>,
    active_preset_id: Option<Uuid>,
    default_presets: HashMap<String, WorkspacePreset>,
}

impl WorkspacePresetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            presets: Arc::new(RwLock::new(HashMap::new())),
            active_preset_id: None,
            default_presets: HashMap::new(),
        };
        manager.register_default_presets();
        manager
    }

    fn register_default_presets(&mut self) {
        let single_terminal =
            WorkspacePreset::new("Single Terminal".to_string(), LayoutType::Single)
                .with_description("A single terminal panel".to_string());
        self.default_presets
            .insert("single".to_string(), single_terminal);

        let split_terminal = {
            let mut preset =
                WorkspacePreset::new("Horizontal Split".to_string(), LayoutType::HorizontalSplit)
                    .with_description("Two terminal panels side by side".to_string());

            let left = PanelConfig::new(PanelType::Terminal, PathBuf::from("."), 0.5, 1.0)
                .with_position(0.0, 0.0)
                .with_title("Left".to_string());

            let right = PanelConfig::new(PanelType::Terminal, PathBuf::from("."), 0.5, 1.0)
                .with_position(0.5, 0.0)
                .with_title("Right".to_string());

            preset.panels.push(left);
            preset.panels.push(right);
            preset.active_panel_id = preset.panels.first().map(|p| p.panel_id);
            preset
        };
        self.default_presets
            .insert("split".to_string(), split_terminal);

        let fullstack = {
            let mut preset = WorkspacePreset::new("Fullstack".to_string(), LayoutType::Grid)
                .with_description("Frontend, backend, and agent panels".to_string());

            let editor = PanelConfig::new(PanelType::Editor, PathBuf::from("./src"), 0.5, 0.6)
                .with_position(0.0, 0.0)
                .with_title("Editor".to_string());

            let terminal = PanelConfig::new(PanelType::Terminal, PathBuf::from("."), 0.5, 0.4)
                .with_position(0.5, 0.0)
                .with_title("Terminal".to_string());

            let preview = PanelConfig::new(PanelType::Preview, PathBuf::new(), 0.5, 0.4)
                .with_position(0.5, 0.4)
                .with_title("Preview".to_string())
                .with_metadata(
                    "url".to_string(),
                    serde_json::json!("http://localhost:3000"),
                );

            let agent_hub = PanelConfig::new(PanelType::AgentHub, PathBuf::new(), 0.5, 0.4)
                .with_position(0.0, 0.6)
                .with_title("Agent Hub".to_string());

            preset.panels.push(editor);
            preset.panels.push(terminal);
            preset.panels.push(preview);
            preset.panels.push(agent_hub);
            preset
        };
        self.default_presets
            .insert("fullstack".to_string(), fullstack);

        let agent_workspace = {
            let mut preset =
                WorkspacePreset::new("Agent Workspace".to_string(), LayoutType::VerticalSplit)
                    .with_description("Multi-agent collaboration workspace".to_string());

            let main_terminal = PanelConfig::new(PanelType::Terminal, PathBuf::from("."), 1.0, 0.4)
                .with_position(0.0, 0.0)
                .with_title("Main Terminal".to_string());

            let agent_hub = PanelConfig::new(PanelType::AgentHub, PathBuf::new(), 1.0, 0.4)
                .with_position(0.0, 0.4)
                .with_title("Agent Hub".to_string());

            let output = PanelConfig::new(PanelType::Output, PathBuf::new(), 1.0, 0.2)
                .with_position(0.0, 0.8)
                .with_title("Output".to_string());

            preset.panels.push(main_terminal);
            preset.panels.push(agent_hub);
            preset.panels.push(output);
            preset
        };
        self.default_presets
            .insert("agent".to_string(), agent_workspace);
    }

    pub fn save_preset(&self, preset: WorkspacePreset) -> Uuid {
        let id = preset.id;
        self.presets.write().insert(id, preset);
        id
    }

    pub fn get_preset(&self, id: &Uuid) -> Option<WorkspacePreset> {
        self.presets.read().get(id).cloned()
    }

    pub fn get_preset_by_name(&self, name: &str) -> Option<WorkspacePreset> {
        let presets = self.presets.read();
        presets.values().find(|p| p.name == name).cloned()
    }

    pub fn get_default_preset(&self, name: &str) -> Option<WorkspacePreset> {
        self.default_presets.get(name).cloned()
    }

    pub fn list_presets(&self) -> Vec<WorkspacePreset> {
        self.presets.read().values().cloned().collect()
    }

    pub fn list_default_preset_names(&self) -> Vec<String> {
        self.default_presets.keys().cloned().collect()
    }

    pub fn delete_preset(&self, id: &Uuid) -> bool {
        self.presets.write().remove(id).is_some()
    }

    pub fn set_active_preset(&mut self, id: Option<Uuid>) {
        self.active_preset_id = id;
    }

    pub fn get_active_preset(&self) -> Option<WorkspacePreset> {
        self.active_preset_id.and_then(|id| self.get_preset(&id))
    }

    pub fn create_from_current_state(
        &self,
        name: String,
        current_panels: Vec<PanelConfig>,
        active_panel_id: Option<Uuid>,
    ) -> WorkspacePreset {
        let mut preset = WorkspacePreset::new(name, LayoutType::Custom);
        preset.panels = current_panels;
        preset.active_panel_id = active_panel_id;
        preset
    }

    pub fn serialize_to_json(&self, preset: &WorkspacePreset) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(preset)
    }

    pub fn deserialize_from_json(&self, json: &str) -> Result<WorkspacePreset, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn export_all_presets(&self) -> Result<String, serde_json::Error> {
        let presets: Vec<WorkspacePreset> = self.list_presets();
        serde_json::to_string_pretty(&presets)
    }

    pub fn import_presets(&self, json: &str) -> Result<Vec<Uuid>, serde_json::Error> {
        let imported: Vec<WorkspacePreset> = serde_json::from_str(json)?;
        let ids: Vec<Uuid> = imported
            .into_iter()
            .map(|preset| {
                let id = preset.id;
                self.presets.write().insert(id, preset);
                id
            })
            .collect();
        Ok(ids)
    }
}

impl Default for WorkspacePresetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for WorkspacePresetManager {
    fn clone(&self) -> Self {
        Self {
            presets: Arc::clone(&self.presets),
            active_preset_id: self.active_preset_id,
            default_presets: self.default_presets.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayoutCommand {
    pub command_type: LayoutCommandType,
    pub preset_name: Option<String>,
    pub preset_id: Option<Uuid>,
    pub custom_layout: Option<WorkspacePreset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutCommandType {
    LoadPreset,
    SaveCurrentAsPreset,
    ListPresets,
    DeletePreset,
    ResetToDefault,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_workspace_preset() {
        let preset = WorkspacePreset::new("Test".to_string(), LayoutType::Single)
            .with_description("Test preset".to_string());

        assert_eq!(preset.name, "Test");
        assert_eq!(preset.layout_type, LayoutType::Single);
    }

    #[test]
    fn test_panel_config() {
        let panel = PanelConfig::new(PanelType::Terminal, PathBuf::from("/home/user"), 0.5, 1.0)
            .with_shell(ShellType::Zsh)
            .with_position(0.5, 0.0);

        assert_eq!(panel.panel_type, PanelType::Terminal);
        assert_eq!(panel.shell_type, Some(ShellType::Zsh));
        assert_eq!(panel.width_ratio, 0.5);
    }

    #[test]
    fn test_preset_manager() {
        let manager = WorkspacePresetManager::new();

        assert!(manager.get_default_preset("single").is_some());
        assert!(manager.get_default_preset("split").is_some());
        assert!(manager.get_default_preset("fullstack").is_some());
    }

    #[test]
    fn test_serialize_deserialize() {
        let manager = WorkspacePresetManager::new();
        let preset = manager.get_default_preset("fullstack").unwrap();

        let json = manager.serialize_to_json(&preset).unwrap();
        let deserialized = manager.deserialize_from_json(&json).unwrap();

        assert_eq!(preset.name, deserialized.name);
        assert_eq!(preset.layout_type, deserialized.layout_type);
        assert_eq!(preset.panels.len(), deserialized.panels.len());
    }
}
