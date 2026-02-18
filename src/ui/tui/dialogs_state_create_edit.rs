use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CreateDialogState {
    pub(super) workspace_name: String,
    pub(super) project_index: usize,
    pub(super) agent: AgentType,
    pub(super) base_branch: String,
    pub(super) setup_commands: String,
    pub(super) auto_run_setup_commands: bool,
    pub(super) start_config: StartAgentConfigState,
    pub(super) focused_field: CreateDialogField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EditDialogState {
    pub(super) workspace_name: String,
    pub(super) workspace_path: PathBuf,
    pub(super) is_main: bool,
    pub(super) branch: String,
    pub(super) base_branch: String,
    pub(super) agent: AgentType,
    pub(super) was_running: bool,
    pub(super) focused_field: EditDialogField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CreateDialogField {
    WorkspaceName,
    Project,
    BaseBranch,
    SetupCommands,
    AutoRunSetupCommands,
    Agent,
    StartConfig(StartAgentConfigField),
    CreateButton,
    CancelButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EditDialogField {
    BaseBranch,
    Agent,
    SaveButton,
    CancelButton,
}

impl EditDialogField {
    pub(super) fn next(self) -> Self {
        match self {
            Self::BaseBranch => Self::Agent,
            Self::Agent => Self::SaveButton,
            Self::SaveButton => Self::CancelButton,
            Self::CancelButton => Self::BaseBranch,
        }
    }

    pub(super) fn previous(self) -> Self {
        match self {
            Self::BaseBranch => Self::CancelButton,
            Self::Agent => Self::BaseBranch,
            Self::SaveButton => Self::Agent,
            Self::CancelButton => Self::SaveButton,
        }
    }
}

impl CreateDialogField {
    pub(super) fn next(self) -> Self {
        match self {
            Self::WorkspaceName => Self::Project,
            Self::Project => Self::BaseBranch,
            Self::BaseBranch => Self::SetupCommands,
            Self::SetupCommands => Self::AutoRunSetupCommands,
            Self::AutoRunSetupCommands => Self::Agent,
            Self::Agent => Self::StartConfig(StartAgentConfigField::Prompt),
            Self::StartConfig(field) => {
                if field == StartAgentConfigField::Unsafe {
                    Self::CreateButton
                } else {
                    Self::StartConfig(field.next())
                }
            }
            Self::CreateButton => Self::CancelButton,
            Self::CancelButton => Self::WorkspaceName,
        }
    }

    pub(super) fn previous(self) -> Self {
        match self {
            Self::WorkspaceName => Self::CancelButton,
            Self::Project => Self::WorkspaceName,
            Self::BaseBranch => Self::Project,
            Self::SetupCommands => Self::BaseBranch,
            Self::AutoRunSetupCommands => Self::SetupCommands,
            Self::Agent => Self::AutoRunSetupCommands,
            Self::StartConfig(field) => {
                if field == StartAgentConfigField::Prompt {
                    Self::Agent
                } else {
                    Self::StartConfig(field.previous())
                }
            }
            Self::CreateButton => Self::StartConfig(StartAgentConfigField::Unsafe),
            Self::CancelButton => Self::CreateButton,
        }
    }

    #[cfg(test)]
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::WorkspaceName => "name",
            Self::Project => "project",
            Self::BaseBranch => "base_branch",
            Self::SetupCommands => "setup_commands",
            Self::AutoRunSetupCommands => "auto_run_setup_commands",
            Self::Agent => "agent",
            Self::StartConfig(field) => field.label(),
            Self::CreateButton => "create",
            Self::CancelButton => "cancel",
        }
    }
}
