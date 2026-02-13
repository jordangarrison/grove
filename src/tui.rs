use ftui::core::event::{Event, KeyCode, KeyEvent, KeyEventKind, Modifiers};
use ftui::core::geometry::Rect;
use ftui::render::frame::Frame;
use ftui::widgets::Widget;
use ftui::widgets::paragraph::Paragraph;
use ftui::{App, Cmd, Model, ScreenMode};

use crate::adapters::{
    BootstrapData, CommandGitAdapter, CommandSystemAdapter, DiscoveryState, PlaceholderTmuxAdapter,
    bootstrap_data,
};
use crate::state::{Action, AppState, PaneFocus, UiMode, reduce};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Msg {
    Quit,
    Action(Action),
    Noop,
}

impl From<Event> for Msg {
    fn from(event: Event) -> Self {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                kind: KeyEventKind::Press,
                ..
            }) => Self::Quit,
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) if modifiers.contains(Modifiers::CTRL) => Self::Quit,
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                kind: KeyEventKind::Press,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Down,
                kind: KeyEventKind::Press,
                ..
            }) => Self::Action(Action::MoveSelectionDown),
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                kind: KeyEventKind::Press,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => Self::Action(Action::MoveSelectionUp),
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                kind: KeyEventKind::Press,
                ..
            }) => Self::Action(Action::ToggleFocus),
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => Self::Action(Action::EnterPreviewMode),
            Event::Key(KeyEvent {
                code: KeyCode::Escape,
                kind: KeyEventKind::Press,
                ..
            }) => Self::Action(Action::EnterListMode),
            _ => Self::Noop,
        }
    }
}

struct GroveApp {
    repo_name: String,
    state: AppState,
    discovery_state: DiscoveryState,
}

impl GroveApp {
    fn new() -> Self {
        let bootstrap = bootstrap_data(
            &CommandGitAdapter,
            &PlaceholderTmuxAdapter,
            &CommandSystemAdapter,
        );
        Self::from_bootstrap(bootstrap)
    }

    fn from_bootstrap(bootstrap: BootstrapData) -> Self {
        Self {
            repo_name: bootstrap.repo_name,
            state: AppState::new(bootstrap.workspaces),
            discovery_state: bootstrap.discovery_state,
        }
    }

    fn mode_label(&self) -> &'static str {
        match self.state.mode {
            UiMode::List => "List",
            UiMode::Preview => "Preview",
        }
    }

    fn focus_label(&self) -> &'static str {
        match self.state.focus {
            PaneFocus::WorkspaceList => "WorkspaceList",
            PaneFocus::Preview => "Preview",
        }
    }

    fn selected_status_hint(&self) -> &'static str {
        match self
            .state
            .selected_workspace()
            .map(|workspace| workspace.status)
        {
            Some(crate::domain::WorkspaceStatus::Main) => "main worktree",
            Some(crate::domain::WorkspaceStatus::Idle) => "idle",
            Some(crate::domain::WorkspaceStatus::Unknown) => "unknown",
            None => "none",
        }
    }

    fn status_bar_line(&self) -> String {
        match &self.discovery_state {
            DiscoveryState::Error(message) => {
                format!("Status: discovery error ({message}) [q]quit")
            }
            DiscoveryState::Empty => "Status: no worktrees found [q]quit".to_string(),
            DiscoveryState::Ready => match self.state.mode {
                UiMode::List => format!(
                    "Status: [j/k]move [Tab]focus [Enter]preview [q]quit | selected={}",
                    self.selected_status_hint()
                ),
                UiMode::Preview => "Status: [Esc]list [Tab]focus [q]quit".to_string(),
            },
        }
    }

    fn shell_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("Grove Shell | Repo: {}", self.repo_name),
            format!(
                "Mode: {} | Focus: {}",
                self.mode_label(),
                self.focus_label()
            ),
            "Workspaces (j/k, arrows, Tab focus, Enter preview, Esc list)".to_string(),
        ];

        match &self.discovery_state {
            DiscoveryState::Error(message) => {
                lines.push(format!("! discovery failed: {message}"));
            }
            DiscoveryState::Empty => {
                lines.push("No workspaces discovered".to_string());
            }
            DiscoveryState::Ready => {
                for (idx, workspace) in self.state.workspaces.iter().enumerate() {
                    let selected = if idx == self.state.selected_index {
                        ">"
                    } else {
                        " "
                    };
                    lines.push(format!(
                        "{} {} {} | {} | {}",
                        selected,
                        workspace.status.icon(),
                        workspace.name,
                        workspace.branch,
                        workspace.path.display()
                    ));
                }
            }
        }

        let selected_workspace = self
            .state
            .selected_workspace()
            .map(|workspace| {
                format!(
                    "{} ({}, {})",
                    workspace.name,
                    workspace.branch,
                    workspace.path.display()
                )
            })
            .unwrap_or_else(|| "none".to_string());

        lines.push(String::new());
        lines.push("Preview Pane".to_string());
        lines.push(format!(
            "Selected workspace: {} (placeholder output)",
            selected_workspace
        ));
        lines.push(self.status_bar_line());

        lines
    }
}

impl Model for GroveApp {
    type Message = Msg;

    fn update(&mut self, msg: Msg) -> Cmd<Self::Message> {
        match msg {
            Msg::Quit => Cmd::Quit,
            Msg::Action(action) => {
                reduce(&mut self.state, action);
                Cmd::None
            }
            Msg::Noop => Cmd::None,
        }
    }

    fn view(&self, frame: &mut Frame) {
        let area = Rect::from_size(frame.buffer.width(), frame.buffer.height());
        let content = self.shell_lines().join("\n");
        Paragraph::new(content).render(area, frame);
    }
}

pub fn run() -> std::io::Result<()> {
    App::new(GroveApp::new())
        .screen_mode(ScreenMode::AltScreen)
        .run()
}

#[cfg(test)]
mod tests {
    use super::{GroveApp, Msg};
    use crate::adapters::{BootstrapData, DiscoveryState};
    use crate::domain::{AgentType, Workspace, WorkspaceStatus};
    use crate::state::Action;
    use ftui::Cmd;
    use ftui::core::event::{Event, KeyCode, KeyEvent, KeyEventKind, Modifiers};
    use std::path::PathBuf;

    fn fixture_app() -> GroveApp {
        GroveApp::from_bootstrap(BootstrapData {
            repo_name: "grove".to_string(),
            workspaces: vec![
                Workspace::try_new(
                    "grove".to_string(),
                    PathBuf::from("/repos/grove"),
                    "main".to_string(),
                    Some(1_700_000_200),
                    AgentType::Claude,
                    WorkspaceStatus::Main,
                    true,
                )
                .expect("workspace should be valid"),
                Workspace::try_new(
                    "feature-a".to_string(),
                    PathBuf::from("/repos/grove-feature-a"),
                    "feature-a".to_string(),
                    Some(1_700_000_100),
                    AgentType::Codex,
                    WorkspaceStatus::Idle,
                    false,
                )
                .expect("workspace should be valid"),
            ],
            discovery_state: DiscoveryState::Ready,
        })
    }

    #[test]
    fn key_q_maps_to_quit() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('q')).with_kind(KeyEventKind::Press));
        assert_eq!(Msg::from(event), Msg::Quit);
    }

    #[test]
    fn ctrl_c_maps_to_quit() {
        let event = Event::Key(
            KeyEvent::new(KeyCode::Char('c'))
                .with_modifiers(Modifiers::CTRL)
                .with_kind(KeyEventKind::Press),
        );
        assert_eq!(Msg::from(event), Msg::Quit);
    }

    #[test]
    fn key_j_maps_to_move_down_action() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('j')).with_kind(KeyEventKind::Press));
        assert_eq!(Msg::from(event), Msg::Action(Action::MoveSelectionDown));
    }

    #[test]
    fn tab_maps_to_toggle_focus_action() {
        let event = Event::Key(KeyEvent::new(KeyCode::Tab).with_kind(KeyEventKind::Press));
        assert_eq!(Msg::from(event), Msg::Action(Action::ToggleFocus));
    }

    #[test]
    fn action_message_updates_model_state() {
        let mut app = fixture_app();
        let cmd = ftui::Model::update(&mut app, Msg::Action(Action::MoveSelectionDown));
        assert!(matches!(cmd, Cmd::None));
        assert_eq!(app.state.selected_index, 1);
    }

    #[test]
    fn shell_contains_list_preview_and_status_placeholders() {
        let app = fixture_app();
        let lines = app.shell_lines();
        let content = lines.join("\n");

        assert!(content.contains("Workspaces"));
        assert!(content.contains("Preview Pane"));
        assert!(content.contains("Status:"));
        assert!(content.contains("feature-a | feature-a | /repos/grove-feature-a"));
    }

    #[test]
    fn shell_renders_discovery_error_state() {
        let app = GroveApp::from_bootstrap(BootstrapData {
            repo_name: "grove".to_string(),
            workspaces: Vec::new(),
            discovery_state: DiscoveryState::Error("fatal: not a git repository".to_string()),
        });
        let lines = app.shell_lines();
        let content = lines.join("\n");

        assert!(content.contains("discovery failed"));
        assert!(content.contains("discovery error"));
    }
}
