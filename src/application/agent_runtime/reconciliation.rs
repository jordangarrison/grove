use std::collections::HashSet;

use crate::domain::Workspace;

use super::sessions::session_name_for_workspace_in_project;
use super::status::detect_status;
use super::{ReconciliationResult, SessionActivity};

pub fn reconcile_with_sessions(
    mut workspaces: Vec<Workspace>,
    running_sessions: &HashSet<String>,
    previously_running_workspace_names: &HashSet<String>,
) -> ReconciliationResult {
    let mut matched_sessions = HashSet::with_capacity(running_sessions.len());

    for workspace in &mut workspaces {
        let session_name = session_name_for_workspace_in_project(
            workspace.project_name.as_deref(),
            &workspace.name,
        );
        let has_live_session = running_sessions.contains(&session_name);
        if has_live_session {
            matched_sessions.insert(session_name.clone());
            workspace.status = detect_status(
                "",
                SessionActivity::Active,
                workspace.is_main,
                true,
                workspace.supported_agent,
                &session_name,
            );
            workspace.is_orphaned = false;
        } else {
            workspace.status = detect_status(
                "",
                SessionActivity::Idle,
                workspace.is_main,
                false,
                workspace.supported_agent,
                &session_name,
            );
            workspace.is_orphaned = if workspace.is_main {
                false
            } else {
                previously_running_workspace_names.contains(&workspace.name)
            };
        }
    }

    let mut orphaned_sessions: Vec<String> = running_sessions
        .iter()
        .filter(|session_name| !matched_sessions.contains(*session_name))
        .cloned()
        .collect();
    orphaned_sessions.sort();

    ReconciliationResult {
        workspaces,
        orphaned_sessions,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use crate::domain::{AgentType, WorkspaceStatus};

    use super::reconcile_with_sessions;

    fn fixture_workspace(name: &str, is_main: bool) -> crate::domain::Workspace {
        crate::domain::Workspace::try_new(
            name.to_string(),
            PathBuf::from(format!("/repos/grove-{name}")),
            if is_main {
                "main".to_string()
            } else {
                name.to_string()
            },
            Some(1_700_000_100),
            AgentType::Claude,
            if is_main {
                WorkspaceStatus::Main
            } else {
                WorkspaceStatus::Idle
            },
            is_main,
        )
        .expect("workspace should be valid")
    }

    #[test]
    fn reconciliation_marks_active_orphaned_and_orphan_sessions() {
        let workspaces = vec![
            fixture_workspace("grove", true),
            fixture_workspace("feature-a", false),
            fixture_workspace("feature-b", false),
        ];

        let running_sessions = HashSet::from([
            "grove-ws-grove".to_string(),
            "grove-ws-feature-a".to_string(),
            "grove-ws-zombie".to_string(),
        ]);
        let previously_running = HashSet::from(["feature-b".to_string()]);

        let result = reconcile_with_sessions(workspaces, &running_sessions, &previously_running);
        assert_eq!(result.workspaces[0].status, WorkspaceStatus::Active);
        assert_eq!(result.workspaces[1].status, WorkspaceStatus::Active);
        assert_eq!(result.workspaces[2].status, WorkspaceStatus::Idle);
        assert!(result.workspaces[2].is_orphaned);
        assert_eq!(
            result.orphaned_sessions,
            vec!["grove-ws-zombie".to_string()]
        );
    }
}
