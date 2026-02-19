use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StartAgentConfigState {
    pub(super) prompt: String,
    pub(super) pre_launch_command: String,
    pub(super) skip_permissions: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StartOptions {
    pub(super) prompt: Option<String>,
    pub(super) pre_launch_command: Option<String>,
    pub(super) skip_permissions: bool,
}

impl StartAgentConfigState {
    pub(super) fn new(prompt: String, pre_launch_command: String, skip_permissions: bool) -> Self {
        Self {
            prompt,
            pre_launch_command,
            skip_permissions,
        }
    }

    pub(super) fn is_input_nonempty(&self) -> bool {
        !self.prompt.is_empty() || !self.pre_launch_command.is_empty()
    }

    pub(super) fn parse_start_options(&self) -> StartOptions {
        StartOptions {
            prompt: trimmed_nonempty(&self.prompt),
            pre_launch_command: trimmed_nonempty(&self.pre_launch_command),
            skip_permissions: self.skip_permissions,
        }
    }

    pub(super) fn backspace(&mut self, field: StartAgentConfigField) {
        match field {
            StartAgentConfigField::Prompt => {
                self.prompt.pop();
            }
            StartAgentConfigField::PreLaunchCommand => {
                self.pre_launch_command.pop();
            }
            StartAgentConfigField::Unsafe => {}
        }
    }

    pub(super) fn clear(&mut self, field: StartAgentConfigField) {
        match field {
            StartAgentConfigField::Prompt => self.prompt.clear(),
            StartAgentConfigField::PreLaunchCommand => self.pre_launch_command.clear(),
            StartAgentConfigField::Unsafe => {}
        }
    }

    pub(super) fn push_char(&mut self, field: StartAgentConfigField, character: char) {
        match field {
            StartAgentConfigField::Prompt => self.prompt.push(character),
            StartAgentConfigField::PreLaunchCommand => self.pre_launch_command.push(character),
            StartAgentConfigField::Unsafe => {}
        }
    }

    pub(super) fn toggle_unsafe(&mut self) {
        self.skip_permissions = !self.skip_permissions;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LaunchDialogState {
    pub(super) start_config: StartAgentConfigState,
    pub(super) focused_field: LaunchDialogField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DeleteDialogState {
    pub(super) project_name: Option<String>,
    pub(super) project_path: Option<PathBuf>,
    pub(super) workspace_name: String,
    pub(super) branch: String,
    pub(super) path: PathBuf,
    pub(super) is_missing: bool,
    pub(super) delete_local_branch: bool,
    pub(super) focused_field: DeleteDialogField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MergeDialogState {
    pub(super) project_name: Option<String>,
    pub(super) project_path: Option<PathBuf>,
    pub(super) workspace_name: String,
    pub(super) workspace_branch: String,
    pub(super) workspace_path: PathBuf,
    pub(super) base_branch: String,
    pub(super) cleanup_workspace: bool,
    pub(super) cleanup_local_branch: bool,
    pub(super) focused_field: MergeDialogField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct UpdateFromBaseDialogState {
    pub(super) project_name: Option<String>,
    pub(super) project_path: Option<PathBuf>,
    pub(super) is_main_workspace: bool,
    pub(super) workspace_name: String,
    pub(super) workspace_branch: String,
    pub(super) workspace_path: PathBuf,
    pub(super) base_branch: String,
    pub(super) focused_field: UpdateFromBaseDialogField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeleteDialogField {
    DeleteLocalBranch,
    DeleteButton,
    CancelButton,
}

impl DeleteDialogField {
    pub(super) fn next(self) -> Self {
        match self {
            Self::DeleteLocalBranch => Self::DeleteButton,
            Self::DeleteButton => Self::CancelButton,
            Self::CancelButton => Self::DeleteLocalBranch,
        }
    }

    pub(super) fn previous(self) -> Self {
        match self {
            Self::DeleteLocalBranch => Self::CancelButton,
            Self::DeleteButton => Self::DeleteLocalBranch,
            Self::CancelButton => Self::DeleteButton,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MergeDialogField {
    CleanupWorkspace,
    CleanupLocalBranch,
    MergeButton,
    CancelButton,
}

impl MergeDialogField {
    pub(super) fn next(self) -> Self {
        match self {
            Self::CleanupWorkspace => Self::CleanupLocalBranch,
            Self::CleanupLocalBranch => Self::MergeButton,
            Self::MergeButton => Self::CancelButton,
            Self::CancelButton => Self::CleanupWorkspace,
        }
    }

    pub(super) fn previous(self) -> Self {
        match self {
            Self::CleanupWorkspace => Self::CancelButton,
            Self::CleanupLocalBranch => Self::CleanupWorkspace,
            Self::MergeButton => Self::CleanupLocalBranch,
            Self::CancelButton => Self::MergeButton,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UpdateFromBaseDialogField {
    UpdateButton,
    CancelButton,
}

impl UpdateFromBaseDialogField {
    pub(super) fn next(self) -> Self {
        match self {
            Self::UpdateButton => Self::CancelButton,
            Self::CancelButton => Self::UpdateButton,
        }
    }

    pub(super) fn previous(self) -> Self {
        self.next()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StartAgentConfigField {
    Prompt,
    PreLaunchCommand,
    Unsafe,
}

impl StartAgentConfigField {
    pub(super) fn next(self) -> Self {
        match self {
            Self::Prompt => Self::PreLaunchCommand,
            Self::PreLaunchCommand => Self::Unsafe,
            Self::Unsafe => Self::Prompt,
        }
    }

    pub(super) fn previous(self) -> Self {
        match self {
            Self::Prompt => Self::Unsafe,
            Self::PreLaunchCommand => Self::Prompt,
            Self::Unsafe => Self::PreLaunchCommand,
        }
    }

    #[cfg(test)]
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Prompt => "prompt",
            Self::PreLaunchCommand => "pre_launch_command",
            Self::Unsafe => "unsafe",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LaunchDialogField {
    StartConfig(StartAgentConfigField),
    StartButton,
    CancelButton,
}

impl LaunchDialogField {
    pub(super) fn next(self) -> Self {
        match self {
            Self::StartConfig(field) => {
                if field == StartAgentConfigField::Unsafe {
                    Self::StartButton
                } else {
                    Self::StartConfig(field.next())
                }
            }
            Self::StartButton => Self::CancelButton,
            Self::CancelButton => Self::StartConfig(StartAgentConfigField::Prompt),
        }
    }

    pub(super) fn previous(self) -> Self {
        match self {
            Self::StartConfig(field) => {
                if field == StartAgentConfigField::Prompt {
                    Self::CancelButton
                } else {
                    Self::StartConfig(field.previous())
                }
            }
            Self::StartButton => Self::StartConfig(StartAgentConfigField::Unsafe),
            Self::CancelButton => Self::StartButton,
        }
    }

    #[cfg(test)]
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::StartConfig(field) => field.label(),
            Self::StartButton => "start",
            Self::CancelButton => "cancel",
        }
    }
}
