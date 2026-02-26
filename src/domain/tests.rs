use super::{
    AgentType, PullRequest, PullRequestStatus, Workspace, WorkspaceStatus, WorkspaceValidationError,
};
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
    assert!(workspace.pull_requests.is_empty());
}

#[test]
fn workspace_accepts_pull_request_metadata() {
    let workspace = Workspace::try_new(
        "feature-x".to_string(),
        PathBuf::from("/repos/grove-feature-x"),
        "feature-x".to_string(),
        None,
        AgentType::Codex,
        WorkspaceStatus::Idle,
        false,
    )
    .expect("workspace should be valid")
    .with_pull_requests(vec![PullRequest {
        number: 42,
        url: "https://github.com/acme/grove/pull/42".to_string(),
        status: PullRequestStatus::Merged,
    }]);

    assert_eq!(workspace.pull_requests.len(), 1);
    assert_eq!(workspace.pull_requests[0].number, 42);
    assert_eq!(workspace.pull_requests[0].status, PullRequestStatus::Merged);
}

#[test]
fn agent_type_metadata_roundtrips_marker() {
    for agent in AgentType::all() {
        assert_eq!(AgentType::from_marker(agent.marker()), Some(*agent));
        assert!(!agent.label().is_empty());
        assert!(!agent.command_override_env_var().is_empty());
    }
}

#[test]
fn agent_type_cycles_all_variants() {
    let mut forward = AgentType::Claude;
    for _ in 0..AgentType::all().len() {
        forward = forward.next();
    }
    assert_eq!(forward, AgentType::Claude);

    let mut backward = AgentType::Claude;
    for _ in 0..AgentType::all().len() {
        backward = backward.previous();
    }
    assert_eq!(backward, AgentType::Claude);
}
