//! Chrome geometry this shell publishes to its endpoint for the `ui.layout` API.
//!
//! Sidebar and tab-bar geometry is client-owned, so the endpoint cannot compute
//! this: it depends on client-local state (collapse mode, section split, scroll
//! offsets, overlay) that never reaches the server. The report is assembled from
//! the same [`ShellHitMap`] the mouse router uses, so reported rects are exactly
//! the rects that were last painted. Empty rects are dropped throughout: the
//! sidebar renderer emits degenerate zero-size hits for rows it clipped away,
//! and a consumer painting chips over them would place them on nothing.

use super::*;

use crate::api::schema::{
    PaneLayoutRect, SidebarMode, UiLayoutAgentEntry, UiLayoutCanvas, UiLayoutPath, UiLayoutSidebar,
    UiLayoutTabBar, UiLayoutTabEntry, UiLayoutWorkspaceEntry,
};
use crate::protocol::endpoint::UiLayoutReport;

impl ClientShellState {
    /// Chrome geometry as of the last [`compose`](ClientShellState::compose).
    ///
    /// `cols`/`rows` must be the size that compose was last called with;
    /// otherwise the layout rects and the hit map describe different frames.
    pub(crate) fn ui_layout_report(&self, cols: u16, rows: u16) -> UiLayoutReport {
        let layout = self.layout(cols, rows);
        let mobile = layout.mobile_header.height > 0;

        UiLayoutReport {
            layout: if mobile {
                UiLayoutPath::Mobile
            } else {
                UiLayoutPath::Desktop
            },
            obscured: self.ui_layout_obscured(),
            canvas: UiLayoutCanvas {
                width: cols,
                height: rows,
            },
            sidebar: self.ui_layout_sidebar(layout.sidebar, mobile),
            tab_bar: self.ui_layout_tab_bar(layout.tab_bar),
        }
    }

    /// Whether a full-canvas overlay is covering the chrome this report describes.
    ///
    /// Only the overlays that paint over the whole frame count; popups, menus and
    /// single-line prompts leave the sidebar and tab bar readable underneath.
    fn ui_layout_obscured(&self) -> bool {
        matches!(
            self.overlay.as_ref().map(ClientShellOverlay::kind),
            Some(
                ClientShellOverlayKind::Onboarding
                    | ClientShellOverlayKind::ReleaseNotes
                    | ClientShellOverlayKind::Settings
                    | ClientShellOverlayKind::Help
                    | ClientShellOverlayKind::Navigator
            )
        )
    }

    fn ui_layout_sidebar(&self, sidebar_rect: Rect, mobile: bool) -> UiLayoutSidebar {
        let rect: PaneLayoutRect = sidebar_rect.into();
        let hidden = |rect| UiLayoutSidebar {
            mode: SidebarMode::Hidden,
            rect,
            workspaces: Vec::new(),
            agents: Vec::new(),
        };

        if mobile || sidebar_rect.width == 0 {
            return hidden(rect);
        }

        let mode = if self.sidebar_collapsed {
            match self.config.sidebar_collapsed_mode {
                SidebarCollapsedModeConfig::Hidden => return hidden(rect),
                SidebarCollapsedModeConfig::Compact => SidebarMode::Compact,
            }
        } else {
            SidebarMode::Expanded
        };

        UiLayoutSidebar {
            mode,
            rect,
            workspaces: self.ui_layout_workspaces(),
            agents: self.ui_layout_agents(),
        }
    }

    /// Workspace rows belonging to the active endpoint.
    ///
    /// With several endpoints connected the sidebar interleaves their workspaces
    /// under machine headers; `ui.layout` describes one machine, so rows from
    /// other endpoints are dropped rather than reported with ambiguous ids.
    fn ui_layout_workspaces(&self) -> Vec<UiLayoutWorkspaceEntry> {
        let focused = self
            .snapshot
            .as_deref()
            .and_then(|snapshot| snapshot.focused_workspace_id.as_deref());
        self.hits
            .workspaces
            .iter()
            .filter(|hit| hit.endpoint_id == self.active_endpoint_id && !hit.rect.is_empty())
            .map(|hit| UiLayoutWorkspaceEntry {
                workspace_id: hit.workspace_id.clone(),
                focused: focused == Some(hit.workspace_id.as_str()),
                rect: hit.rect.into(),
            })
            .collect()
    }

    /// Agent rows belonging to the active endpoint, in painted order.
    ///
    /// The single-endpoint sidebar fills `hits.agents`; the multi-endpoint one
    /// fills `hits.endpoint_agents` instead, so both are read and only one ever
    /// contributes.
    fn ui_layout_agents(&self) -> Vec<UiLayoutAgentEntry> {
        let entry = |rect: Rect, pane_id: &String| UiLayoutAgentEntry {
            pane_id: pane_id.clone(),
            rect: rect.into(),
        };
        self.hits
            .agents
            .iter()
            .map(|(rect, pane_id)| entry(*rect, pane_id))
            .chain(
                self.hits
                    .endpoint_agents
                    .iter()
                    .filter(|(_, endpoint_id, _)| *endpoint_id == self.active_endpoint_id)
                    .map(|(rect, _, pane_id)| entry(*rect, pane_id)),
            )
            .filter(|entry| entry.rect.width > 0 && entry.rect.height > 0)
            .collect()
    }

    fn ui_layout_tab_bar(&self, tab_bar_rect: Rect) -> UiLayoutTabBar {
        let snapshot = self.snapshot.as_deref();
        let workspace_id = snapshot
            .and_then(|snapshot| snapshot.focused_workspace_id.clone())
            .unwrap_or_default();
        let focused_tab_id = snapshot.and_then(|snapshot| snapshot.focused_tab_id.as_deref());

        // A hidden tab bar leaves stale hits behind; report no tabs rather than
        // rects that are no longer painted.
        let tabs = if tab_bar_rect.height == 0 {
            Vec::new()
        } else {
            self.hits
                .tabs
                .iter()
                .filter(|(rect, _)| !rect.is_empty())
                .map(|(rect, tab_id)| UiLayoutTabEntry {
                    tab_id: tab_id.clone(),
                    focused: focused_tab_id == Some(tab_id.as_str()),
                    rect: (*rect).into(),
                })
                .collect()
        };

        UiLayoutTabBar {
            rect: tab_bar_rect.into(),
            workspace_id,
            tabs,
        }
    }
}
