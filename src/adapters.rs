use std::collections::HashSet;

use crate::domain::{Workspace, WorkspaceStatus};

pub trait GitAdapter {
    fn list_workspaces(&self) -> Vec<Workspace>;
}

pub trait TmuxAdapter {
    fn running_workspaces(&self) -> HashSet<String>;
}

pub trait SystemAdapter {
    fn repo_name(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapData {
    pub repo_name: String,
    pub workspaces: Vec<Workspace>,
}

pub fn bootstrap_data(
    git: &impl GitAdapter,
    tmux: &impl TmuxAdapter,
    system: &impl SystemAdapter,
) -> BootstrapData {
    let running = tmux.running_workspaces();
    let mut workspaces = git.list_workspaces();
    for workspace in &mut workspaces {
        if workspace.is_main {
            workspace.status = WorkspaceStatus::Main;
            continue;
        }
        workspace.status = if running.contains(&workspace.name) {
            WorkspaceStatus::Unknown
        } else {
            WorkspaceStatus::Idle
        };
    }

    BootstrapData {
        repo_name: system.repo_name(),
        workspaces,
    }
}

pub struct PlaceholderGitAdapter;

impl GitAdapter for PlaceholderGitAdapter {
    fn list_workspaces(&self) -> Vec<Workspace> {
        vec![
            Workspace::try_new(
                "grove".to_string(),
                "main".to_string(),
                crate::domain::AgentType::Claude,
                WorkspaceStatus::Main,
                true,
            )
            .expect("main workspace should be valid"),
            Workspace::try_new(
                "phase-1-sample".to_string(),
                "phase-1-sample".to_string(),
                crate::domain::AgentType::Codex,
                WorkspaceStatus::Idle,
                false,
            )
            .expect("workspace should be valid"),
        ]
    }
}

pub struct PlaceholderTmuxAdapter;

impl TmuxAdapter for PlaceholderTmuxAdapter {
    fn running_workspaces(&self) -> HashSet<String> {
        HashSet::new()
    }
}

pub struct PlaceholderSystemAdapter;

impl SystemAdapter for PlaceholderSystemAdapter {
    fn repo_name(&self) -> String {
        "grove".to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{BootstrapData, GitAdapter, SystemAdapter, TmuxAdapter, bootstrap_data};
    use crate::domain::{AgentType, Workspace, WorkspaceStatus};

    struct FakeGitAdapter;

    impl GitAdapter for FakeGitAdapter {
        fn list_workspaces(&self) -> Vec<Workspace> {
            vec![
                Workspace::try_new(
                    "grove".to_string(),
                    "main".to_string(),
                    AgentType::Claude,
                    WorkspaceStatus::Main,
                    true,
                )
                .expect("workspace should be valid"),
                Workspace::try_new(
                    "feature-a".to_string(),
                    "feature-a".to_string(),
                    AgentType::Codex,
                    WorkspaceStatus::Idle,
                    false,
                )
                .expect("workspace should be valid"),
                Workspace::try_new(
                    "feature-b".to_string(),
                    "feature-b".to_string(),
                    AgentType::Claude,
                    WorkspaceStatus::Idle,
                    false,
                )
                .expect("workspace should be valid"),
            ]
        }
    }

    struct FakeTmuxAdapter;

    impl TmuxAdapter for FakeTmuxAdapter {
        fn running_workspaces(&self) -> HashSet<String> {
            HashSet::from(["feature-a".to_string()])
        }
    }

    struct FakeSystemAdapter;

    impl SystemAdapter for FakeSystemAdapter {
        fn repo_name(&self) -> String {
            "repo-x".to_string()
        }
    }

    #[test]
    fn bootstrap_data_uses_adapter_contracts_deterministically() {
        let data: BootstrapData =
            bootstrap_data(&FakeGitAdapter, &FakeTmuxAdapter, &FakeSystemAdapter);

        assert_eq!(data.repo_name, "repo-x");
        assert_eq!(data.workspaces.len(), 3);
        assert_eq!(data.workspaces[0].status, WorkspaceStatus::Main);
        assert_eq!(data.workspaces[1].status, WorkspaceStatus::Unknown);
        assert_eq!(data.workspaces[2].status, WorkspaceStatus::Idle);
    }
}
