use ftui::core::event::{Event, KeyCode, KeyEvent, KeyEventKind, Modifiers};
use ftui::core::geometry::Rect;
use ftui::render::frame::Frame;
use ftui::widgets::Widget;
use ftui::widgets::paragraph::Paragraph;
use ftui::{App, Cmd, Model, ScreenMode};

use crate::adapters::{
    PlaceholderGitAdapter, PlaceholderSystemAdapter, PlaceholderTmuxAdapter, bootstrap_data,
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
}

impl GroveApp {
    fn new() -> Self {
        let bootstrap = bootstrap_data(
            &PlaceholderGitAdapter,
            &PlaceholderTmuxAdapter,
            &PlaceholderSystemAdapter,
        );
        Self {
            repo_name: bootstrap.repo_name,
            state: AppState::new(bootstrap.workspaces),
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

        for (idx, workspace) in self.state.workspaces.iter().enumerate() {
            let selected = if idx == self.state.selected_index {
                ">"
            } else {
                " "
            };
            lines.push(format!(
                "{} {} {} [{}]",
                selected,
                workspace.status.icon(),
                workspace.name,
                workspace.agent.label()
            ));
        }

        let selected_workspace = self
            .state
            .selected_workspace()
            .map(|workspace| workspace.name.clone())
            .unwrap_or_else(|| "none".to_string());

        lines.push(String::new());
        lines.push("Preview Pane".to_string());
        lines.push(format!(
            "Selected workspace: {} (placeholder output)",
            selected_workspace
        ));
        lines.push("Status Bar: [q]quit [Tab]focus [Enter]preview [Esc]list".to_string());

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
    use crate::state::Action;
    use ftui::Cmd;
    use ftui::core::event::{Event, KeyCode, KeyEvent, KeyEventKind, Modifiers};

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
        let mut app = GroveApp::new();
        let cmd = ftui::Model::update(&mut app, Msg::Action(Action::MoveSelectionDown));
        assert!(matches!(cmd, Cmd::None));
        assert_eq!(app.state.selected_index, 1);
    }

    #[test]
    fn shell_contains_list_preview_and_status_placeholders() {
        let app = GroveApp::new();
        let lines = app.shell_lines();
        let content = lines.join("\n");

        assert!(content.contains("Workspaces"));
        assert!(content.contains("Preview Pane"));
        assert!(content.contains("Status Bar"));
    }
}
