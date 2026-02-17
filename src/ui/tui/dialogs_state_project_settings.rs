use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectDialogState {
    pub(super) filter: String,
    pub(super) filtered_project_indices: Vec<usize>,
    pub(super) selected_filtered_index: usize,
    pub(super) add_dialog: Option<ProjectAddDialogState>,
    pub(super) defaults_dialog: Option<ProjectDefaultsDialogState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectAddDialogState {
    pub(super) name: String,
    pub(super) path: String,
    pub(super) focused_field: ProjectAddDialogField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectDefaultsDialogState {
    pub(super) project_index: usize,
    pub(super) base_branch: String,
    pub(super) setup_commands: String,
    pub(super) auto_run_setup_commands: bool,
    pub(super) focused_field: ProjectDefaultsDialogField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProjectAddDialogField {
    Name,
    Path,
    AddButton,
    CancelButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProjectDefaultsDialogField {
    BaseBranch,
    SetupCommands,
    AutoRunSetupCommands,
    SaveButton,
    CancelButton,
}

impl ProjectAddDialogField {
    pub(super) fn next(self) -> Self {
        match self {
            Self::Name => Self::Path,
            Self::Path => Self::AddButton,
            Self::AddButton => Self::CancelButton,
            Self::CancelButton => Self::Name,
        }
    }

    pub(super) fn previous(self) -> Self {
        match self {
            Self::Name => Self::CancelButton,
            Self::Path => Self::Name,
            Self::AddButton => Self::Path,
            Self::CancelButton => Self::AddButton,
        }
    }
}

impl ProjectDefaultsDialogField {
    pub(super) fn next(self) -> Self {
        match self {
            Self::BaseBranch => Self::SetupCommands,
            Self::SetupCommands => Self::AutoRunSetupCommands,
            Self::AutoRunSetupCommands => Self::SaveButton,
            Self::SaveButton => Self::CancelButton,
            Self::CancelButton => Self::BaseBranch,
        }
    }

    pub(super) fn previous(self) -> Self {
        match self {
            Self::BaseBranch => Self::CancelButton,
            Self::SetupCommands => Self::BaseBranch,
            Self::AutoRunSetupCommands => Self::SetupCommands,
            Self::SaveButton => Self::AutoRunSetupCommands,
            Self::CancelButton => Self::SaveButton,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SettingsDialogState {
    pub(super) multiplexer: MultiplexerKind,
    pub(super) focused_field: SettingsDialogField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SettingsDialogField {
    Multiplexer,
    SaveButton,
    CancelButton,
}

impl SettingsDialogField {
    pub(super) fn next(self) -> Self {
        match self {
            Self::Multiplexer => Self::SaveButton,
            Self::SaveButton => Self::CancelButton,
            Self::CancelButton => Self::Multiplexer,
        }
    }

    pub(super) fn previous(self) -> Self {
        match self {
            Self::Multiplexer => Self::CancelButton,
            Self::SaveButton => Self::Multiplexer,
            Self::CancelButton => Self::SaveButton,
        }
    }
}
