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
    .expect("workspace should be valid")
    .with_base_branch(Some("main".to_string()))
    .with_orphaned(true)
    .with_supported_agent(false);

    assert_eq!(workspace.agent.label(), "Codex");
    assert_eq!(workspace.path, PathBuf::from("/repos/grove-feature-x"));
    assert_eq!(workspace.base_branch.as_deref(), Some("main"));
    assert!(workspace.is_orphaned);
    assert!(!workspace.supported_agent);
}
