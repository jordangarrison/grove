use crate::domain::Workspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneFocus {
    WorkspaceList,
    Preview,
}

impl PaneFocus {
    pub fn label(self) -> &'static str {
        match self {
            Self::WorkspaceList => "WorkspaceList",
            Self::Preview => "Preview",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::WorkspaceList => "workspace_list",
            Self::Preview => "preview",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    List,
    Preview,
}

impl UiMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::List => "List",
            Self::Preview => "Preview",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Preview => "preview",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    MoveSelectionUp,
    MoveSelectionDown,
    ToggleFocus,
    EnterPreviewMode,
    EnterListMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub workspaces: Vec<Workspace>,
    pub selected_index: usize,
    pub focus: PaneFocus,
    pub mode: UiMode,
}

impl AppState {
    pub fn new(workspaces: Vec<Workspace>) -> Self {
        Self {
            workspaces,
            selected_index: 0,
            focus: PaneFocus::WorkspaceList,
            mode: UiMode::List,
        }
    }

    pub fn selected_workspace(&self) -> Option<&Workspace> {
        self.workspaces.get(self.selected_index)
    }

    #[cfg(test)]
    pub fn selected_workspace_mut(&mut self) -> Option<&mut Workspace> {
        self.workspaces.get_mut(self.selected_index)
    }
}

pub fn reduce(state: &mut AppState, action: Action) {
    match action {
        Action::MoveSelectionUp => {
            if state.selected_index > 0 {
                state.selected_index -= 1;
            }
        }
        Action::MoveSelectionDown => {
            let last = state.workspaces.len().saturating_sub(1);
            if state.selected_index < last {
                state.selected_index += 1;
            }
        }
        Action::ToggleFocus => {
            state.focus = match state.focus {
                PaneFocus::WorkspaceList => PaneFocus::Preview,
                PaneFocus::Preview => PaneFocus::WorkspaceList,
            };
        }
        Action::EnterPreviewMode => {
            if state.selected_workspace().is_some() {
                state.mode = UiMode::Preview;
                state.focus = PaneFocus::Preview;
            }
        }
        Action::EnterListMode => {
            state.mode = UiMode::List;
            state.focus = PaneFocus::WorkspaceList;
        }
    }
}

#[cfg(test)]
mod tests;
