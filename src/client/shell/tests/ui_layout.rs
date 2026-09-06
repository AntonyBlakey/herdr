//! Coverage for the `ui.layout` report, re-homed from the fork's
//! `src/app/api/ui.rs` tests.
//!
//! Those drove `compute_view` over `AppState` sidebar geometry. Upstream moved
//! that geometry into this client shell, so the same behaviours are asserted
//! against the shell's own hit map instead.

use super::*;
use crate::api::schema::{SidebarMode, UiLayoutPath};
use crate::config::SidebarCollapsedModeConfig;
use crate::protocol::endpoint::UiLayoutReport;
use crate::protocol::ClientShellAgent;

/// A snapshot with `count` workspaces, each with one tab and one agent pane.
fn snapshot_with(workspace_count: usize) -> ClientShellSnapshot {
    let mut snapshot = snapshot();
    snapshot.workspaces.clear();
    snapshot.tabs.clear();
    snapshot.panes.clear();
    snapshot.agents.clear();
    snapshot.agent_order.clear();

    for index in 0..workspace_count {
        let ordinal = index + 1;
        let workspace_id = format!("ws_{ordinal}");
        let tab_id = format!("tab_{ordinal}");
        let pane_id = format!("pane_{ordinal}");
        let focused = index == 0;
        snapshot.workspaces.push(ClientShellWorkspace {
            workspace_id: workspace_id.clone(),
            active_tab_id: tab_id.clone(),
            new_workspace_cwd: "/repo".into(),
            number: ordinal,
            label: format!("workspace-{ordinal}"),
            custom_label: false,
            branch: None,
            git_ahead_behind: None,
            tokens: Vec::new(),
            worktree: None,
            focused,
            agent_status: AgentStatus::Idle,
        });
        snapshot.tabs.push(ClientShellTab {
            tab_id: tab_id.clone(),
            workspace_id: workspace_id.clone(),
            number: 1,
            label: "1".into(),
            custom_label: false,
            zoomed: false,
            focused,
            agent_status: AgentStatus::Idle,
        });
        snapshot.panes.push(ClientShellPane {
            pane_id: pane_id.clone(),
            workspace_id: workspace_id.clone(),
            tab_id: tab_id.clone(),
            label: None,
            cwd: Some("/repo".into()),
            foreground_cwd: Some("/repo".into()),
            focused,
            right_click_passthrough: false,
        });
        snapshot
            .agents
            .push(agent(&pane_id, &workspace_id, &tab_id, focused));
        snapshot.agent_order.push(pane_id);
    }

    let first = snapshot.workspaces.first();
    snapshot.focused_workspace_id = first.map(|workspace| workspace.workspace_id.clone());
    snapshot.focused_tab_id = first.map(|workspace| workspace.active_tab_id.clone());
    snapshot.focused_pane_id = snapshot.panes.first().map(|pane| pane.pane_id.clone());
    snapshot
}

fn agent(pane_id: &str, workspace_id: &str, tab_id: &str, focused: bool) -> ClientShellAgent {
    ClientShellAgent {
        pane_id: pane_id.into(),
        workspace_id: workspace_id.into(),
        tab_id: tab_id.into(),
        name: Some(pane_id.into()),
        display_agent: Some("claude".into()),
        agent: Some("claude".into()),
        title: None,
        terminal_title: None,
        terminal_title_stripped: None,
        agent_status: AgentStatus::Working,
        state_change_seq: 1,
        state_labels: Vec::new(),
        tokens: Vec::new(),
        focused,
    }
}

/// Adds `count` extra agent panes to the first workspace's first tab.
fn add_agents(snapshot: &mut ClientShellSnapshot, count: usize) {
    let (workspace_id, tab_id) = {
        let workspace = &snapshot.workspaces[0];
        (
            workspace.workspace_id.clone(),
            workspace.active_tab_id.clone(),
        )
    };
    for index in 0..count {
        let pane_id = format!("extra_pane_{index}");
        snapshot.panes.push(ClientShellPane {
            pane_id: pane_id.clone(),
            workspace_id: workspace_id.clone(),
            tab_id: tab_id.clone(),
            label: None,
            cwd: Some("/repo".into()),
            foreground_cwd: Some("/repo".into()),
            focused: false,
            right_click_passthrough: false,
        });
        snapshot
            .agents
            .push(agent(&pane_id, &workspace_id, &tab_id, false));
        snapshot.agent_order.push(pane_id);
    }
}

fn shell(snapshot: ClientShellSnapshot) -> ClientShellState {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot));
    state.set_pane_surface(surface());
    state
}

/// Compose at `cols`x`rows`, then report the geometry that compose produced.
fn report_at(state: &mut ClientShellState, cols: u16, rows: u16) -> UiLayoutReport {
    state.compose(cols, rows);
    state.ui_layout_report(cols, rows)
}

#[test]
fn expanded_sidebar_reports_workspace_and_agent_ids() {
    let mut state = shell(snapshot_with(1));

    let report = report_at(&mut state, 100, 30);

    assert_eq!(report.layout, UiLayoutPath::Desktop);
    assert_eq!(report.canvas.width, 100);
    assert_eq!(report.canvas.height, 30);
    assert_eq!(report.sidebar.mode, SidebarMode::Expanded);
    assert_eq!(report.sidebar.workspaces.len(), 1);
    assert_eq!(report.sidebar.workspaces[0].workspace_id, "ws_1");
    assert!(report.sidebar.workspaces[0].focused);
    assert_eq!(report.sidebar.agents.len(), 1);
    assert_eq!(report.sidebar.agents[0].pane_id, "pane_1");
}

#[test]
fn expanded_sidebar_reports_visual_order_and_focus() {
    let mut state = shell(snapshot_with(3));

    let report = report_at(&mut state, 100, 30);

    let ids: Vec<&str> = report
        .sidebar
        .workspaces
        .iter()
        .map(|entry| entry.workspace_id.as_str())
        .collect();
    assert_eq!(ids, ["ws_1", "ws_2", "ws_3"]);
    assert!(report.sidebar.workspaces[0].focused);
    assert!(!report.sidebar.workspaces[1].focused);
    assert!(!report.sidebar.workspaces[2].focused);
}

#[test]
fn compact_sidebar_reports_flat_one_row_rects() {
    let mut state = shell(snapshot_with(2));
    state.config.sidebar_collapsed_mode = SidebarCollapsedModeConfig::Compact;
    state.sidebar_collapsed = true;

    let report = report_at(&mut state, 100, 30);

    assert_eq!(report.sidebar.mode, SidebarMode::Compact);
    assert_eq!(report.sidebar.workspaces.len(), 2);
    for entry in &report.sidebar.workspaces {
        assert_eq!(entry.rect.height, 1);
    }
    assert_ne!(
        report.sidebar.workspaces[0].rect.y,
        report.sidebar.workspaces[1].rect.y
    );
    assert_eq!(report.sidebar.agents.len(), 2);
    assert_eq!(report.sidebar.agents[0].rect.height, 1);
}

#[test]
fn hidden_sidebar_reports_empty_lists_and_zero_rect() {
    let mut state = shell(snapshot_with(1));
    state.config.sidebar_collapsed_mode = SidebarCollapsedModeConfig::Hidden;
    state.sidebar_collapsed = true;

    let report = report_at(&mut state, 100, 30);

    assert_eq!(report.sidebar.mode, SidebarMode::Hidden);
    assert_eq!(report.sidebar.rect.width, 0);
    assert!(report.sidebar.workspaces.is_empty());
    assert!(report.sidebar.agents.is_empty());
}

#[test]
fn mobile_layout_hides_sidebar_but_keeps_tab_bar_workspace_id() {
    let mut state = shell(snapshot_with(1));

    let report = report_at(&mut state, 44, 20);

    assert_eq!(report.layout, UiLayoutPath::Mobile);
    assert_eq!(report.sidebar.mode, SidebarMode::Hidden);
    assert_eq!(report.sidebar.rect.width, 0);
    assert!(report.sidebar.workspaces.is_empty());
    assert!(report.sidebar.agents.is_empty());
    assert_eq!(report.tab_bar.rect.width, 0);
    assert!(report.tab_bar.tabs.is_empty());
    assert_eq!(report.tab_bar.workspace_id, "ws_1");
}

#[test]
fn tab_bar_absent_when_single_tab_hidden_by_config() {
    let mut state = shell(snapshot_with(1));
    state.config.hide_tab_bar_when_single_tab = true;

    let report = report_at(&mut state, 100, 30);

    assert_eq!(report.tab_bar.rect.height, 0);
    assert!(report.tab_bar.tabs.is_empty());
    assert_eq!(report.tab_bar.workspace_id, "ws_1");
}

#[test]
fn tab_bar_absent_on_short_terminal() {
    let mut state = shell(snapshot_with(1));

    let report = report_at(&mut state, 100, 1);

    assert_eq!(report.tab_bar.rect.height, 0);
    assert!(report.tab_bar.tabs.is_empty());
}

#[test]
fn tab_bar_reports_focused_tab_and_ids_in_order() {
    let mut snapshot = snapshot_with(1);
    for ordinal in 2..=3 {
        snapshot.tabs.push(ClientShellTab {
            tab_id: format!("tab_1_{ordinal}"),
            workspace_id: "ws_1".into(),
            number: ordinal,
            label: format!("{ordinal}"),
            custom_label: false,
            zoomed: false,
            focused: false,
            agent_status: AgentStatus::Idle,
        });
    }
    snapshot.tabs[1].focused = true;
    snapshot.tabs[0].focused = false;
    snapshot.focused_tab_id = Some("tab_1_2".into());
    let mut state = shell(snapshot);

    let report = report_at(&mut state, 100, 30);

    let ids: Vec<&str> = report
        .tab_bar
        .tabs
        .iter()
        .map(|entry| entry.tab_id.as_str())
        .collect();
    assert_eq!(ids, ["tab_1", "tab_1_2", "tab_1_3"]);
    assert!(!report.tab_bar.tabs[0].focused);
    assert!(report.tab_bar.tabs[1].focused);
    assert!(!report.tab_bar.tabs[2].focused);
}

#[test]
fn obscured_true_only_under_full_canvas_overlays() {
    let mut state = shell(snapshot_with(1));
    state.compose(100, 30);

    assert!(!state.ui_layout_report(100, 30).obscured);

    for overlay in [
        ClientShellOverlay::Onboarding,
        ClientShellOverlay::ReleaseNotes(crate::app::state::ReleaseNotesState {
            version: "0.0.0".into(),
            body: String::new(),
            scroll: 0,
            preview: false,
        }),
    ] {
        state.overlay = Some(overlay);
        assert!(
            state.ui_layout_report(100, 30).obscured,
            "{:?} should obscure",
            state.overlay.as_ref().map(ClientShellOverlay::kind)
        );
    }

    state.overlay = Some(ClientShellOverlay::ProductAnnouncement(
        crate::app::state::ProductAnnouncementState {
            version: "0.0.0".into(),
            id: "announcement".into(),
            title: "Title".into(),
            body: String::new(),
            scroll: 0,
            preview: false,
        },
    ));
    assert!(
        !state.ui_layout_report(100, 30).obscured,
        "a product announcement is a popup, not a full-canvas overlay"
    );
}

#[test]
fn expanded_agent_panel_clips_and_honours_scroll() {
    let mut snapshot = snapshot_with(1);
    add_agents(&mut snapshot, 8);
    let mut state = shell(snapshot);

    let full = report_at(&mut state, 100, 40);
    let clipped = report_at(&mut state, 100, 12);

    assert!(
        clipped.sidebar.agents.len() < full.sidebar.agents.len(),
        "expected clipping on a short terminal: {} of {}",
        clipped.sidebar.agents.len(),
        full.sidebar.agents.len()
    );
    assert!(!clipped.sidebar.agents.is_empty());

    let first_unscrolled = clipped.sidebar.agents[0].pane_id.clone();
    state.agent_scroll = 1;
    let scrolled = report_at(&mut state, 100, 12);
    assert_ne!(scrolled.sidebar.agents[0].pane_id, first_unscrolled);
}

#[test]
fn compact_workspace_list_clips_to_available_rows() {
    let mut state = shell(snapshot_with(20));
    state.config.sidebar_collapsed_mode = SidebarCollapsedModeConfig::Compact;
    state.sidebar_collapsed = true;

    let report = report_at(&mut state, 100, 10);

    assert!(
        report.sidebar.workspaces.len() < 20,
        "test setup should force clipping, got {}",
        report.sidebar.workspaces.len()
    );
    assert!(!report.sidebar.workspaces.is_empty());
}

#[test]
fn zero_height_terminal_does_not_panic() {
    let mut state = shell(snapshot_with(1));

    let report = report_at(&mut state, 80, 0);

    assert_eq!(report.canvas.height, 0);
    assert!(report.sidebar.workspaces.is_empty());
    assert!(report.tab_bar.tabs.is_empty());
}

#[test]
fn no_workspaces_reports_empty_tab_bar() {
    let mut state = shell(snapshot_with(0));

    let report = report_at(&mut state, 100, 30);

    assert!(report.tab_bar.workspace_id.is_empty());
    assert!(report.tab_bar.tabs.is_empty());
    assert!(report.sidebar.workspaces.is_empty());
}

#[test]
fn rects_reported_are_the_rects_that_were_painted() {
    let mut state = shell(snapshot_with(2));

    let report = report_at(&mut state, 100, 30);

    for entry in &report.sidebar.workspaces {
        let hit = state
            .hits
            .workspaces
            .iter()
            .find(|hit| hit.workspace_id == entry.workspace_id)
            .expect("reported workspace must come from the hit map");
        assert_eq!(entry.rect.x, hit.rect.x);
        assert_eq!(entry.rect.y, hit.rect.y);
        assert_eq!(entry.rect.width, hit.rect.width);
        assert_eq!(entry.rect.height, hit.rect.height);
    }
    assert_eq!(
        report.sidebar.rect.width,
        state.layout(100, 30).sidebar.width
    );
    assert_eq!(
        report.tab_bar.rect.height,
        state.layout(100, 30).tab_bar.height
    );
}
