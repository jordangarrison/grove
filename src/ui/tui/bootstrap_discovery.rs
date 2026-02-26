use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use crate::domain::{PullRequest, PullRequestStatus, Workspace};
use crate::infrastructure::adapters::{
    BootstrapData, CommandGitAdapter, CommandMultiplexerAdapter, CommandSystemAdapter,
    DiscoveryState, MultiplexerAdapter, bootstrap_data,
};
use crate::infrastructure::config::ProjectConfig;

#[derive(Debug, Clone)]
struct StaticMultiplexerAdapter {
    running_sessions: HashSet<String>,
}

impl MultiplexerAdapter for StaticMultiplexerAdapter {
    fn running_sessions(&self) -> HashSet<String> {
        self.running_sessions.clone()
    }
}

#[derive(Debug)]
enum PullRequestLookupError {
    CommandUnavailable,
    CommandFailed,
    DecodeFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct GitHubPullRequestRecord {
    number: u64,
    url: String,
    state: String,
    #[serde(rename = "mergedAt")]
    merged_at: Option<String>,
}

pub(super) fn bootstrap_data_for_projects(projects: &[ProjectConfig]) -> BootstrapData {
    if projects.is_empty() {
        return BootstrapData {
            repo_name: "grove".to_string(),
            workspaces: Vec::new(),
            discovery_state: DiscoveryState::Empty,
        };
    }

    let live_multiplexer = CommandMultiplexerAdapter;
    let static_multiplexer = StaticMultiplexerAdapter {
        running_sessions: live_multiplexer.running_sessions(),
    };
    let mut workspaces = Vec::new();
    let mut errors = Vec::new();
    for project in projects {
        let git = CommandGitAdapter::for_repo(project.path.clone());
        let system = CommandSystemAdapter::for_repo(project.path.clone());
        let bootstrap = bootstrap_data(&git, &static_multiplexer, &system);
        if let DiscoveryState::Error(message) = &bootstrap.discovery_state {
            errors.push(format!("{}: {message}", project.name));
        }

        let mut project_workspaces = bootstrap.workspaces;
        attach_pull_requests_to_workspaces(project.path.as_path(), &mut project_workspaces);
        workspaces.extend(project_workspaces);
    }

    let discovery_state = if !workspaces.is_empty() {
        DiscoveryState::Ready
    } else if !errors.is_empty() {
        DiscoveryState::Error(errors.join("; "))
    } else {
        DiscoveryState::Empty
    };
    let repo_name = if projects.len() == 1 {
        projects[0].name.clone()
    } else {
        format!("{} projects", projects.len())
    };

    BootstrapData {
        repo_name,
        workspaces,
        discovery_state,
    }
}

fn attach_pull_requests_to_workspaces(project_path: &Path, workspaces: &mut [Workspace]) {
    let mut unique_branches = workspaces
        .iter()
        .filter(|workspace| !workspace.is_main)
        .map(|workspace| workspace.branch.trim())
        .filter(|branch| !branch.is_empty() && *branch != "(detached)")
        .map(str::to_string)
        .collect::<Vec<String>>();
    unique_branches.sort();
    unique_branches.dedup();

    let mut by_branch: HashMap<String, Vec<PullRequest>> = HashMap::new();
    for branch in unique_branches {
        let pull_requests = match list_pull_requests_for_branch(project_path, branch.as_str()) {
            Ok(pull_requests) => pull_requests,
            Err(PullRequestLookupError::CommandUnavailable) => return,
            Err(PullRequestLookupError::CommandFailed | PullRequestLookupError::DecodeFailed) => {
                Vec::new()
            }
        };
        by_branch.insert(branch, pull_requests);
    }

    for workspace in workspaces {
        if workspace.is_main {
            workspace.pull_requests.clear();
        } else {
            workspace.pull_requests = by_branch
                .get(&workspace.branch)
                .cloned()
                .unwrap_or_default();
        }
    }
}

fn list_pull_requests_for_branch(
    project_path: &Path,
    branch: &str,
) -> Result<Vec<PullRequest>, PullRequestLookupError> {
    let output = Command::new("gh")
        .current_dir(project_path)
        .args([
            "pr",
            "list",
            "--head",
            branch,
            "--state",
            "all",
            "--json",
            "number,url,state,mergedAt",
        ])
        .output()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                PullRequestLookupError::CommandUnavailable
            } else {
                PullRequestLookupError::CommandFailed
            }
        })?;

    if !output.status.success() {
        return Err(PullRequestLookupError::CommandFailed);
    }

    let records: Vec<GitHubPullRequestRecord> = serde_json::from_slice(output.stdout.as_slice())
        .map_err(|_| PullRequestLookupError::DecodeFailed)?;
    Ok(records
        .into_iter()
        .map(record_to_pull_request)
        .collect::<Vec<PullRequest>>())
}

fn record_to_pull_request(record: GitHubPullRequestRecord) -> PullRequest {
    PullRequest {
        number: record.number,
        url: record.url,
        status: pull_request_status(record.state.as_str(), record.merged_at.as_deref()),
    }
}

fn pull_request_status(state: &str, merged_at: Option<&str>) -> PullRequestStatus {
    if merged_at.is_some_and(|value| !value.trim().is_empty()) {
        return PullRequestStatus::Merged;
    }
    if state == "OPEN" {
        PullRequestStatus::Open
    } else {
        PullRequestStatus::Closed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pull_request_status_prefers_merged_at_signal() {
        assert_eq!(pull_request_status("OPEN", None), PullRequestStatus::Open);
        assert_eq!(
            pull_request_status("OPEN", Some("2025-01-02T12:34:56Z")),
            PullRequestStatus::Merged
        );
        assert_eq!(
            pull_request_status("CLOSED", None),
            PullRequestStatus::Closed
        );
    }

    #[test]
    fn record_conversion_keeps_number_url_and_status() {
        let record = GitHubPullRequestRecord {
            number: 99,
            url: "https://github.com/acme/grove/pull/99".to_string(),
            state: "MERGED".to_string(),
            merged_at: Some("2025-01-02T12:34:56Z".to_string()),
        };

        let pull_request = record_to_pull_request(record);
        assert_eq!(pull_request.number, 99);
        assert_eq!(pull_request.url, "https://github.com/acme/grove/pull/99");
        assert_eq!(pull_request.status, PullRequestStatus::Merged);
    }
}
