use ratatui::layout::Rect;

use crate::api::schema::{
    PaneLayoutRect, ResponseResult, SidebarMode, UiLayoutAgentEntry, UiLayoutCanvas, UiLayoutPath,
    UiLayoutSidebar, UiLayoutTabBar, UiLayoutTabEntry, UiLayoutWorkspaceEntry,
};
use crate::app::state::ViewLayout;
use crate::app::{App, Mode};
use crate::config::SidebarCollapsedModeConfig;

use super::responses::encode_success;

impl App {
    pub(super) fn handle_ui_layout(&mut self, id: String) -> String {
        let layout = match self.state.view.layout {
            ViewLayout::Desktop => UiLayoutPath::Desktop,
            ViewLayout::Mobile => UiLayoutPath::Mobile,
        };
        let obscured = matches!(
            self.state.mode,
            Mode::Onboarding
                | Mode::ReleaseNotes
                | Mode::Settings
                | Mode::KeybindHelp
                | Mode::Navigator
        );
        let canvas = UiLayoutCanvas {
            width: self.state.view.frame_area.width,
            height: self.state.view.frame_area.height,
        };
        let sidebar = self.ui_layout_sidebar();
        let tab_bar = self.ui_layout_tab_bar();

        encode_success(
            id,
            ResponseResult::UiLayout {
                layout,
                obscured,
                canvas,
                sidebar,
                tab_bar,
            },
        )
    }

    fn ui_layout_sidebar(&self) -> UiLayoutSidebar {
        let sidebar_rect = self.state.view.sidebar_rect;
        let rect: PaneLayoutRect = sidebar_rect.into();

        if self.state.view.layout != ViewLayout::Desktop {
            return UiLayoutSidebar {
                mode: SidebarMode::Hidden,
                rect,
                workspaces: Vec::new(),
                agents: Vec::new(),
            };
        }

        if !self.state.sidebar_collapsed {
            return UiLayoutSidebar {
                mode: SidebarMode::Expanded,
                rect,
                workspaces: self.ui_layout_expanded_workspaces(sidebar_rect),
                agents: self.ui_layout_expanded_agents(sidebar_rect),
            };
        }

        match self.state.sidebar_collapsed_mode {
            SidebarCollapsedModeConfig::Hidden => UiLayoutSidebar {
                mode: SidebarMode::Hidden,
                rect,
                workspaces: Vec::new(),
                agents: Vec::new(),
            },
            SidebarCollapsedModeConfig::Compact => UiLayoutSidebar {
                mode: SidebarMode::Compact,
                rect,
                workspaces: self.ui_layout_compact_workspaces(sidebar_rect),
                agents: self.ui_layout_compact_agents(sidebar_rect),
            },
        }
    }

    fn ui_layout_expanded_workspaces(&self, sidebar_rect: Rect) -> Vec<UiLayoutWorkspaceEntry> {
        crate::ui::compute_workspace_card_areas(&self.state, sidebar_rect)
            .into_iter()
            .map(|card| UiLayoutWorkspaceEntry {
                workspace_id: self.public_workspace_id(card.ws_idx),
                focused: Some(card.ws_idx) == self.state.active,
                rect: card.rect.into(),
            })
            .collect()
    }

    fn ui_layout_expanded_agents(&self, sidebar_rect: Rect) -> Vec<UiLayoutAgentEntry> {
        let (_, detail_area) =
            crate::ui::expanded_sidebar_sections(sidebar_rect, self.state.sidebar_section_split);
        let metrics = crate::ui::agent_panel_scroll_metrics(&self.state, detail_area);
        let body = crate::ui::agent_panel_body_rect(
            detail_area,
            crate::ui::should_show_scrollbar(metrics),
        );
        if body.width == 0 || body.height == 0 {
            return Vec::new();
        }

        let entries = crate::ui::agent_panel_entries(&self.state);
        let mut result = Vec::new();
        let mut row_y = body.y;
        let body_bottom = body.y + body.height;
        for (index, entry) in entries
            .iter()
            .enumerate()
            .skip(self.state.agent_panel_scroll)
        {
            let height = crate::ui::agent_entry_height_in_body(&self.state, entry, body.height);
            if row_y.saturating_add(height) > body_bottom {
                break;
            }
            result.push(UiLayoutAgentEntry {
                pane_id: self
                    .public_pane_id(entry.ws_idx, entry.pane_id)
                    .unwrap_or_default(),
                rect: Rect::new(body.x, row_y, body.width, height).into(),
            });
            row_y = row_y
                .saturating_add(height)
                .saturating_add(crate::ui::agent_entry_gap(
                    &self.state,
                    index,
                    entries.len(),
                ))
                .min(body_bottom);
        }
        result
    }

    fn ui_layout_compact_workspaces(&self, sidebar_rect: Rect) -> Vec<UiLayoutWorkspaceEntry> {
        let (ws_area, _, _) = crate::ui::collapsed_sidebar_sections(sidebar_rect);
        if ws_area == Rect::default() {
            return Vec::new();
        }

        let mut result = Vec::new();
        for idx in 0..self.state.workspaces.len() {
            let y = ws_area.y + idx as u16;
            if y >= ws_area.y + ws_area.height {
                break;
            }
            result.push(UiLayoutWorkspaceEntry {
                workspace_id: self.public_workspace_id(idx),
                focused: Some(idx) == self.state.active,
                rect: Rect::new(ws_area.x, y, ws_area.width, 1).into(),
            });
        }
        result
    }

    fn ui_layout_compact_agents(&self, sidebar_rect: Rect) -> Vec<UiLayoutAgentEntry> {
        let (_, _, detail_area) = crate::ui::collapsed_sidebar_sections(sidebar_rect);
        let detail_content_area = Rect::new(
            detail_area.x,
            detail_area.y,
            detail_area.width,
            detail_area.height.saturating_sub(1),
        );
        if detail_content_area == Rect::default() {
            return Vec::new();
        }

        let entries = crate::ui::agent_panel_entries(&self.state);
        let mut result = Vec::new();
        for (idx, entry) in entries.iter().enumerate() {
            let y = detail_content_area.y + idx as u16;
            if y >= detail_content_area.y + detail_content_area.height {
                break;
            }
            result.push(UiLayoutAgentEntry {
                pane_id: self
                    .public_pane_id(entry.ws_idx, entry.pane_id)
                    .unwrap_or_default(),
                rect: Rect::new(detail_content_area.x, y, detail_content_area.width, 1).into(),
            });
        }
        result
    }

    fn ui_layout_tab_bar(&self) -> UiLayoutTabBar {
        let Some((active_ws_idx, ws)) = self
            .state
            .active
            .and_then(|idx| self.state.workspaces.get(idx).map(|ws| (idx, ws)))
        else {
            return UiLayoutTabBar {
                rect: Rect::default().into(),
                workspace_id: String::new(),
                tabs: Vec::new(),
            };
        };

        let workspace_id = self.public_workspace_id(active_ws_idx);
        let active_tab = ws.active_tab;
        let tabs = self
            .state
            .view
            .tab_hit_areas
            .iter()
            .enumerate()
            .filter(|(_, rect)| rect.width > 0)
            .filter_map(|(idx, rect)| {
                Some(UiLayoutTabEntry {
                    tab_id: self.public_tab_id(active_ws_idx, idx)?,
                    focused: idx == active_tab,
                    rect: (*rect).into(),
                })
            })
            .collect();

        UiLayoutTabBar {
            rect: self.state.view.tab_bar_rect.into(),
            workspace_id,
            tabs,
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Direction;

    use super::*;
    use crate::api::schema::SuccessResponse;
    use crate::app::state::Mode;
    use crate::config::Config;
    use crate::detect::{Agent, AgentState};
    use crate::workspace::Workspace;

    fn test_app(workspace_count: usize) -> App {
        let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new(
            &Config::default(),
            true,
            None,
            api_rx,
            crate::api::EventHub::default(),
        );
        app.state.workspaces = (0..workspace_count)
            .map(|idx| Workspace::test_new(&format!("ws{idx}")))
            .collect();
        if workspace_count > 0 {
            app.state.active = Some(0);
        }
        app.state.mode = Mode::Terminal;
        mark_all_panes_as_agents(&mut app);
        app
    }

    fn mark_all_panes_as_agents(app: &mut App) {
        app.state.ensure_test_terminals();
        let terminal_ids: Vec<_> = app
            .state
            .workspaces
            .iter()
            .flat_map(|ws| ws.tabs.iter())
            .flat_map(|tab| tab.panes.values())
            .map(|pane| pane.attached_terminal_id.clone())
            .collect();
        for terminal_id in terminal_ids {
            if let Some(terminal) = app.state.terminals.get_mut(&terminal_id) {
                terminal.set_detected_state(Some(Agent::Claude), AgentState::Working);
            }
        }
    }

    struct LayoutResponse {
        layout: UiLayoutPath,
        obscured: bool,
        canvas: UiLayoutCanvas,
        sidebar: UiLayoutSidebar,
        tab_bar: UiLayoutTabBar,
    }

    fn layout_response(app: &mut App) -> LayoutResponse {
        let response = app.handle_ui_layout("req".into());
        let success: SuccessResponse = serde_json::from_str(&response).unwrap();
        let ResponseResult::UiLayout {
            layout,
            obscured,
            canvas,
            sidebar,
            tab_bar,
        } = success.result
        else {
            panic!("expected ui_layout response");
        };
        LayoutResponse {
            layout,
            obscured,
            canvas,
            sidebar,
            tab_bar,
        }
    }

    #[test]
    fn expanded_sidebar_reports_workspace_and_agent_ids() {
        let mut app = test_app(1);
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 30));
        let expected_ws_id = app.public_workspace_id(0);
        let root_pane = app.state.workspaces[0].tabs[0].root_pane;
        let expected_pane_id = app.public_pane_id(0, root_pane).unwrap();

        let response = layout_response(&mut app);

        assert_eq!(response.layout, UiLayoutPath::Desktop);
        assert_eq!(
            response.canvas,
            UiLayoutCanvas {
                width: 100,
                height: 30
            }
        );
        assert_eq!(response.sidebar.mode, SidebarMode::Expanded);
        assert_eq!(response.sidebar.workspaces.len(), 1);
        assert_eq!(response.sidebar.workspaces[0].workspace_id, expected_ws_id);
        assert!(response.sidebar.workspaces[0].focused);
        assert_eq!(response.sidebar.agents.len(), 1);
        assert_eq!(response.sidebar.agents[0].pane_id, expected_pane_id);
    }

    #[test]
    fn expanded_sidebar_reports_visual_order_and_focus() {
        let mut app = test_app(3);
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 30));
        let expected_ids: Vec<String> = (0..3).map(|idx| app.public_workspace_id(idx)).collect();

        let response = layout_response(&mut app);

        let actual_ids: Vec<String> = response
            .sidebar
            .workspaces
            .iter()
            .map(|entry| entry.workspace_id.clone())
            .collect();
        assert_eq!(actual_ids, expected_ids);
        assert!(response.sidebar.workspaces[0].focused);
        assert!(!response.sidebar.workspaces[1].focused);
        assert!(!response.sidebar.workspaces[2].focused);
    }

    #[test]
    fn compact_sidebar_reports_flat_one_row_rects() {
        let mut app = test_app(2);
        app.state.sidebar_collapsed = true;
        app.state.sidebar_collapsed_mode = SidebarCollapsedModeConfig::Compact;
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 30));

        let response = layout_response(&mut app);

        assert_eq!(response.sidebar.mode, SidebarMode::Compact);
        assert_eq!(response.sidebar.workspaces.len(), 2);
        for entry in &response.sidebar.workspaces {
            assert_eq!(entry.rect.height, 1);
        }
        assert_eq!(response.sidebar.agents.len(), 2);
        assert_eq!(response.sidebar.agents[0].rect.height, 1);
        assert_ne!(
            response.sidebar.workspaces[0].rect.y,
            response.sidebar.workspaces[1].rect.y
        );
    }

    #[test]
    fn hidden_sidebar_reports_empty_lists_and_zero_rect() {
        let mut app = test_app(1);
        app.state.sidebar_collapsed = true;
        app.state.sidebar_collapsed_mode = SidebarCollapsedModeConfig::Hidden;
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 30));

        let response = layout_response(&mut app);

        assert_eq!(response.sidebar.mode, SidebarMode::Hidden);
        assert_eq!(response.sidebar.rect.width, 0);
        assert!(response.sidebar.workspaces.is_empty());
        assert!(response.sidebar.agents.is_empty());
    }

    #[test]
    fn mobile_layout_hides_sidebar_but_keeps_tab_bar_workspace_id() {
        let mut app = test_app(1);
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 44, 20));
        let expected_ws_id = app.public_workspace_id(0);

        let response = layout_response(&mut app);

        assert_eq!(response.layout, UiLayoutPath::Mobile);
        assert_eq!(response.sidebar.mode, SidebarMode::Hidden);
        assert_eq!(response.sidebar.rect.width, 0);
        assert!(response.sidebar.workspaces.is_empty());
        assert!(response.sidebar.agents.is_empty());
        assert_eq!(response.tab_bar.rect.width, 0);
        assert!(response.tab_bar.tabs.is_empty());
        assert_eq!(response.tab_bar.workspace_id, expected_ws_id);
    }

    #[test]
    fn tab_bar_absent_when_single_tab_hidden_by_config() {
        let mut app = test_app(1);
        app.state.hide_tab_bar_when_single_tab = true;
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 30));
        let expected_ws_id = app.public_workspace_id(0);

        let response = layout_response(&mut app);

        assert_eq!(response.tab_bar.rect.width, 0);
        assert!(response.tab_bar.tabs.is_empty());
        assert_eq!(response.tab_bar.workspace_id, expected_ws_id);
    }

    #[test]
    fn tab_bar_absent_on_short_terminal() {
        let mut app = test_app(1);
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 1));

        let response = layout_response(&mut app);

        assert_eq!(response.tab_bar.rect.width, 0);
        assert!(response.tab_bar.tabs.is_empty());
    }

    #[test]
    fn tab_bar_reports_focused_tab_and_ids_in_order() {
        let mut app = test_app(1);
        app.state.workspaces[0].test_add_tab(Some("logs"));
        app.state.workspaces[0].test_add_tab(Some("build"));
        app.state.workspaces[0].active_tab = 1;
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 30));
        let expected_tab_ids: Vec<String> = (0..3)
            .map(|idx| app.public_tab_id(0, idx).unwrap())
            .collect();

        let response = layout_response(&mut app);

        let actual_tab_ids: Vec<String> = response
            .tab_bar
            .tabs
            .iter()
            .map(|entry| entry.tab_id.clone())
            .collect();
        assert_eq!(actual_tab_ids, expected_tab_ids);
        assert!(response.tab_bar.tabs[1].focused);
        assert!(!response.tab_bar.tabs[0].focused);
        assert!(!response.tab_bar.tabs[2].focused);
    }

    #[test]
    fn obscured_true_only_under_full_canvas_overlay_modes() {
        let mut app = test_app(1);
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 30));

        for mode in [
            Mode::Onboarding,
            Mode::ReleaseNotes,
            Mode::Settings,
            Mode::KeybindHelp,
            Mode::Navigator,
        ] {
            app.state.mode = mode;
            assert!(
                layout_response(&mut app).obscured,
                "{mode:?} should obscure"
            );
        }

        for mode in [
            Mode::Terminal,
            Mode::Navigate,
            Mode::ConfirmClose,
            Mode::ProductAnnouncement,
            Mode::RenameWorkspace,
        ] {
            app.state.mode = mode;
            assert!(
                !layout_response(&mut app).obscured,
                "{mode:?} should not obscure"
            );
        }
    }

    #[test]
    fn expanded_agent_panel_clips_and_honours_scroll() {
        let mut app = test_app(1);
        for _ in 0..4 {
            app.state.workspaces[0].test_split(Direction::Horizontal);
        }
        mark_all_panes_as_agents(&mut app);
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 40));
        let full = layout_response(&mut app);
        assert_eq!(full.sidebar.agents.len(), 5);

        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 12));
        let clipped = layout_response(&mut app);
        assert!(
            clipped.sidebar.agents.len() < 5,
            "expected clipping on a short terminal, got {}",
            clipped.sidebar.agents.len()
        );
        assert!(!clipped.sidebar.agents.is_empty());

        let first_pane_id_unscrolled = clipped.sidebar.agents[0].pane_id.clone();
        app.state.agent_panel_scroll = 1;
        let scrolled = layout_response(&mut app);
        assert_ne!(scrolled.sidebar.agents[0].pane_id, first_pane_id_unscrolled);
    }

    #[test]
    fn compact_workspace_list_clips_to_available_rows() {
        let mut app = test_app(20);
        app.state.sidebar_collapsed = true;
        app.state.sidebar_collapsed_mode = SidebarCollapsedModeConfig::Compact;
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 10));

        let response = layout_response(&mut app);

        let (ws_area, _, _) = crate::ui::collapsed_sidebar_sections(app.state.view.sidebar_rect);
        let expected = (ws_area.height as usize).min(20);
        assert_eq!(response.sidebar.workspaces.len(), expected);
        assert!(expected < 20, "test setup should force clipping");
    }

    #[test]
    fn zero_height_terminal_does_not_panic() {
        let mut app = test_app(1);
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 80, 0));

        let response = layout_response(&mut app);

        assert!(response.sidebar.workspaces.is_empty());
        assert!(response.tab_bar.tabs.is_empty());
        assert_eq!(response.canvas.height, 0);
    }

    #[test]
    fn no_workspaces_reports_empty_tab_bar() {
        let mut app = test_app(0);
        crate::ui::compute_view(&mut app.state, Rect::new(0, 0, 100, 30));

        let response = layout_response(&mut app);

        assert!(response.tab_bar.workspace_id.is_empty());
        assert!(response.tab_bar.tabs.is_empty());
    }
}
