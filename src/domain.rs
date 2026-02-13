use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    Claude,
    Codex,
}

impl AgentType {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Claude => "Claude",
            Self::Codex => "Codex",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceStatus {
    Main,
    Idle,
    Unknown,
}

impl WorkspaceStatus {
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Main => "◉",
            Self::Idle => "○",
            Self::Unknown => "?",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub name: String,
    pub path: PathBuf,
    pub branch: String,
    pub last_activity_unix_secs: Option<i64>,
    pub agent: AgentType,
    pub status: WorkspaceStatus,
    pub is_main: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceValidationError {
    EmptyName,
    EmptyPath,
    EmptyBranch,
    MainWorkspaceMustUseMainStatus,
}

impl Workspace {
    pub fn try_new(
        name: String,
        path: PathBuf,
        branch: String,
        last_activity_unix_secs: Option<i64>,
        agent: AgentType,
        status: WorkspaceStatus,
        is_main: bool,
    ) -> Result<Self, WorkspaceValidationError> {
        if name.trim().is_empty() {
            return Err(WorkspaceValidationError::EmptyName);
        }
        if path.as_os_str().is_empty() {
            return Err(WorkspaceValidationError::EmptyPath);
        }
        if branch.trim().is_empty() {
            return Err(WorkspaceValidationError::EmptyBranch);
        }
        if is_main && status != WorkspaceStatus::Main {
            return Err(WorkspaceValidationError::MainWorkspaceMustUseMainStatus);
        }

        Ok(Self {
            name,
            path,
            branch,
            last_activity_unix_secs,
            agent,
            status,
            is_main,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentType, Workspace, WorkspaceStatus, WorkspaceValidationError};
    use std::path::PathBuf;

    #[test]
    fn main_workspace_requires_main_status() {
        let workspace = Workspace::try_new(
            "grove".to_string(),
            PathBuf::from("/repos/grove"),
            "main".to_string(),
            Some(1_700_000_000),
            AgentType::Claude,
            WorkspaceStatus::Idle,
            true,
        );
        assert_eq!(
            workspace,
            Err(WorkspaceValidationError::MainWorkspaceMustUseMainStatus)
        );
    }

    #[test]
    fn workspace_requires_non_empty_name_and_branch() {
        assert_eq!(
            Workspace::try_new(
                "".to_string(),
                PathBuf::from("/repos/grove"),
                "main".to_string(),
                Some(1_700_000_000),
                AgentType::Claude,
                WorkspaceStatus::Idle,
                false
            ),
            Err(WorkspaceValidationError::EmptyName)
        );
        assert_eq!(
            Workspace::try_new(
                "feature-x".to_string(),
                PathBuf::from("/repos/grove-feature-x"),
                "".to_string(),
                Some(1_700_000_000),
                AgentType::Claude,
                WorkspaceStatus::Idle,
                false
            ),
            Err(WorkspaceValidationError::EmptyBranch)
        );
        assert_eq!(
            Workspace::try_new(
                "feature-x".to_string(),
                PathBuf::new(),
                "feature-x".to_string(),
                Some(1_700_000_000),
                AgentType::Claude,
                WorkspaceStatus::Idle,
                false
            ),
            Err(WorkspaceValidationError::EmptyPath)
        );
    }

    #[test]
    fn workspace_accepts_valid_values() {
        let workspace = Workspace::try_new(
            "feature-x".to_string(),
            PathBuf::from("/repos/grove-feature-x"),
            "feature-x".to_string(),
            None,
            AgentType::Codex,
            WorkspaceStatus::Unknown,
            false,
        )
        .expect("workspace should be valid");

        assert_eq!(workspace.agent.label(), "Codex");
        assert_eq!(workspace.status.icon(), "?");
        assert_eq!(workspace.path, PathBuf::from("/repos/grove-feature-x"));
    }
}
