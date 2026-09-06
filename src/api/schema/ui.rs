use serde::{Deserialize, Serialize};

use super::panes::PaneLayoutRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UiLayoutPath {
    Desktop,
    Mobile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SidebarMode {
    Expanded,
    Compact,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UiLayoutCanvas {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UiLayoutWorkspaceEntry {
    pub workspace_id: String,
    pub focused: bool,
    pub rect: PaneLayoutRect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UiLayoutAgentEntry {
    pub pane_id: String,
    pub rect: PaneLayoutRect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UiLayoutSidebar {
    pub mode: SidebarMode,
    pub rect: PaneLayoutRect,
    pub workspaces: Vec<UiLayoutWorkspaceEntry>,
    pub agents: Vec<UiLayoutAgentEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UiLayoutTabEntry {
    pub tab_id: String,
    pub focused: bool,
    pub rect: PaneLayoutRect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UiLayoutTabBar {
    pub rect: PaneLayoutRect,
    pub workspace_id: String,
    pub tabs: Vec<UiLayoutTabEntry>,
}
