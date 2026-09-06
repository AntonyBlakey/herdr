use crate::api::schema::ResponseResult;
use crate::app::App;

use super::responses::encode_success;

impl App {
    /// Answer `ui.layout` from the foreground shell's last published geometry.
    ///
    /// The server owns none of this — sidebar and tab-bar rects are client
    /// state — so the handler replays the report rather than computing one. A
    /// server with no attached shell answers with an empty desktop canvas, which
    /// is a well-formed envelope carrying no entries.
    pub(super) fn handle_ui_layout(&mut self, id: String) -> String {
        let report = self.state.ui_layout.clone().unwrap_or_default();
        encode_success(
            id,
            ResponseResult::UiLayout {
                layout: report.layout,
                obscured: report.obscured,
                canvas: report.canvas,
                sidebar: report.sidebar,
                tab_bar: report.tab_bar,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::schema::{
        PaneLayoutRect, SidebarMode, SuccessResponse, UiLayoutCanvas, UiLayoutPath,
        UiLayoutSidebar, UiLayoutTabBar, UiLayoutTabEntry, UiLayoutWorkspaceEntry,
    };
    use crate::protocol::endpoint::UiLayoutReport;

    fn test_app() -> App {
        let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
        App::new(
            &crate::config::Config::default(),
            crate::app::AppPolicy::TEST,
            None,
            api_rx,
            crate::api::EventHub::default(),
        )
    }

    fn layout_response(app: &mut App) -> SuccessResponse {
        serde_json::from_str(&app.handle_ui_layout("req".into())).expect("ui_layout response")
    }

    #[test]
    fn no_shell_has_reported_yet_answers_an_empty_envelope() {
        let mut app = test_app();

        let ResponseResult::UiLayout {
            layout,
            obscured,
            canvas,
            sidebar,
            tab_bar,
        } = layout_response(&mut app).result
        else {
            panic!("expected a ui_layout response");
        };

        assert_eq!(layout, UiLayoutPath::Desktop);
        assert!(!obscured);
        assert_eq!(
            canvas,
            UiLayoutCanvas {
                width: 0,
                height: 0
            }
        );
        assert_eq!(sidebar.mode, SidebarMode::Hidden);
        assert!(sidebar.workspaces.is_empty());
        assert!(sidebar.agents.is_empty());
        assert!(tab_bar.workspace_id.is_empty());
        assert!(tab_bar.tabs.is_empty());
    }

    #[test]
    fn a_reported_layout_is_replayed_verbatim() {
        let rect = |x, y, width, height| PaneLayoutRect {
            x,
            y,
            width,
            height,
        };
        let report = UiLayoutReport {
            layout: UiLayoutPath::Mobile,
            obscured: true,
            canvas: UiLayoutCanvas {
                width: 100,
                height: 30,
            },
            sidebar: UiLayoutSidebar {
                mode: SidebarMode::Compact,
                rect: rect(0, 0, 4, 30),
                workspaces: vec![UiLayoutWorkspaceEntry {
                    workspace_id: "ws_1".into(),
                    focused: true,
                    rect: rect(0, 1, 4, 1),
                }],
                agents: Vec::new(),
            },
            tab_bar: UiLayoutTabBar {
                rect: rect(4, 0, 96, 1),
                workspace_id: "ws_1".into(),
                tabs: vec![UiLayoutTabEntry {
                    tab_id: "tab_1".into(),
                    focused: true,
                    rect: rect(4, 0, 10, 1),
                }],
            },
        };
        let mut app = test_app();
        app.state.ui_layout = Some(report.clone());

        let ResponseResult::UiLayout {
            layout,
            obscured,
            canvas,
            sidebar,
            tab_bar,
        } = layout_response(&mut app).result
        else {
            panic!("expected a ui_layout response");
        };

        assert_eq!(layout, report.layout);
        assert_eq!(obscured, report.obscured);
        assert_eq!(canvas, report.canvas);
        assert_eq!(sidebar, report.sidebar);
        assert_eq!(tab_bar, report.tab_bar);
    }
}
