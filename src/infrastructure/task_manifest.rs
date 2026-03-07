use crate::domain::{AgentType, PullRequest, PullRequestStatus, Task, WorkspaceStatus, Worktree};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TaskManifest {
    name: String,
    slug: String,
    root_path: String,
    branch: String,
    worktrees: Vec<TaskManifestWorktree>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TaskManifestWorktree {
    repository_name: String,
    repository_path: String,
    path: String,
    branch: String,
    base_branch: Option<String>,
    last_activity_unix_secs: Option<i64>,
    agent: String,
    status: String,
    is_orphaned: bool,
    supported_agent: bool,
    pull_requests: Vec<TaskManifestPullRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TaskManifestPullRequest {
    number: u64,
    url: String,
    status: String,
}

pub fn encode_task_manifest(task: &Task) -> Result<String, String> {
    let manifest = TaskManifest {
        name: task.name.clone(),
        slug: task.slug.clone(),
        root_path: task.root_path.to_string_lossy().into_owned(),
        branch: task.branch.clone(),
        worktrees: task
            .worktrees
            .iter()
            .map(|worktree| TaskManifestWorktree {
                repository_name: worktree.repository_name.clone(),
                repository_path: worktree.repository_path.to_string_lossy().into_owned(),
                path: worktree.path.to_string_lossy().into_owned(),
                branch: worktree.branch.clone(),
                base_branch: worktree.base_branch.clone(),
                last_activity_unix_secs: worktree.last_activity_unix_secs,
                agent: worktree.agent.marker().to_string(),
                status: workspace_status_name(worktree.status).to_string(),
                is_orphaned: worktree.is_orphaned,
                supported_agent: worktree.supported_agent,
                pull_requests: worktree
                    .pull_requests
                    .iter()
                    .map(|pull_request| TaskManifestPullRequest {
                        number: pull_request.number,
                        url: pull_request.url.clone(),
                        status: pull_request_status_name(pull_request.status).to_string(),
                    })
                    .collect(),
            })
            .collect(),
    };

    toml::to_string_pretty(&manifest)
        .map_err(|error| format!("task manifest encode failed: {error}"))
}

pub fn decode_task_manifest(raw: &str) -> Result<Task, String> {
    let manifest = toml::from_str::<TaskManifest>(raw)
        .map_err(|error| format!("task manifest parse failed: {error}"))?;
    let worktrees = manifest
        .worktrees
        .into_iter()
        .map(decode_worktree)
        .collect::<Result<Vec<Worktree>, String>>()?;

    Task::try_new(
        manifest.name,
        manifest.slug,
        manifest.root_path.into(),
        manifest.branch,
        worktrees,
    )
    .map_err(|error| format!("task manifest invalid: {error:?}"))
}

fn decode_worktree(manifest: TaskManifestWorktree) -> Result<Worktree, String> {
    let agent = AgentType::from_marker(manifest.agent.as_str())
        .ok_or_else(|| format!("unsupported agent '{}'", manifest.agent))?;
    let status = parse_workspace_status(manifest.status.as_str())
        .ok_or_else(|| format!("unsupported workspace status '{}'", manifest.status))?;
    let pull_requests = manifest
        .pull_requests
        .into_iter()
        .map(|pull_request| {
            let status =
                parse_pull_request_status(pull_request.status.as_str()).ok_or_else(|| {
                    format!("unsupported pull request status '{}'", pull_request.status)
                })?;
            Ok(PullRequest {
                number: pull_request.number,
                url: pull_request.url,
                status,
            })
        })
        .collect::<Result<Vec<PullRequest>, String>>()?;

    let worktree = Worktree::try_new(
        manifest.repository_name,
        manifest.repository_path.into(),
        manifest.path.into(),
        manifest.branch,
        agent,
        status,
    )
    .map_err(|error| format!("task worktree invalid: {error:?}"))?;

    Ok(worktree
        .with_base_branch(manifest.base_branch)
        .with_last_activity_unix_secs(manifest.last_activity_unix_secs)
        .with_orphaned(manifest.is_orphaned)
        .with_supported_agent(manifest.supported_agent)
        .with_pull_requests(pull_requests))
}

fn workspace_status_name(status: WorkspaceStatus) -> &'static str {
    match status {
        WorkspaceStatus::Main => "main",
        WorkspaceStatus::Idle => "idle",
        WorkspaceStatus::Active => "active",
        WorkspaceStatus::Thinking => "thinking",
        WorkspaceStatus::Waiting => "waiting",
        WorkspaceStatus::Done => "done",
        WorkspaceStatus::Error => "error",
        WorkspaceStatus::Unknown => "unknown",
        WorkspaceStatus::Unsupported => "unsupported",
    }
}

fn parse_workspace_status(value: &str) -> Option<WorkspaceStatus> {
    match value {
        "main" => Some(WorkspaceStatus::Main),
        "idle" => Some(WorkspaceStatus::Idle),
        "active" => Some(WorkspaceStatus::Active),
        "thinking" => Some(WorkspaceStatus::Thinking),
        "waiting" => Some(WorkspaceStatus::Waiting),
        "done" => Some(WorkspaceStatus::Done),
        "error" => Some(WorkspaceStatus::Error),
        "unknown" => Some(WorkspaceStatus::Unknown),
        "unsupported" => Some(WorkspaceStatus::Unsupported),
        _ => None,
    }
}

fn pull_request_status_name(status: PullRequestStatus) -> &'static str {
    match status {
        PullRequestStatus::Open => "open",
        PullRequestStatus::Merged => "merged",
        PullRequestStatus::Closed => "closed",
    }
}

fn parse_pull_request_status(value: &str) -> Option<PullRequestStatus> {
    match value {
        "open" => Some(PullRequestStatus::Open),
        "merged" => Some(PullRequestStatus::Merged),
        "closed" => Some(PullRequestStatus::Closed),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_task_manifest, encode_task_manifest};
    use crate::domain::{AgentType, Task, WorkspaceStatus, Worktree};
    use std::path::PathBuf;

    fn fixture_task() -> Task {
        let app_worktree = Worktree::try_new(
            "flohome".to_string(),
            PathBuf::from("/repos/flohome"),
            PathBuf::from("/tmp/.grove/tasks/flohome-launch/flohome"),
            "flohome-launch".to_string(),
            AgentType::Codex,
            WorkspaceStatus::Active,
        )
        .expect("worktree should be valid")
        .with_base_branch(Some("main".to_string()));
        let infra_worktree = Worktree::try_new(
            "terraform-fastly".to_string(),
            PathBuf::from("/repos/terraform-fastly"),
            PathBuf::from("/tmp/.grove/tasks/flohome-launch/terraform-fastly"),
            "flohome-launch".to_string(),
            AgentType::Claude,
            WorkspaceStatus::Idle,
        )
        .expect("worktree should be valid")
        .with_base_branch(Some("main".to_string()));
        Task::try_new(
            "flohome-launch".to_string(),
            "flohome-launch".to_string(),
            PathBuf::from("/tmp/.grove/tasks/flohome-launch"),
            "flohome-launch".to_string(),
            vec![app_worktree, infra_worktree],
        )
        .expect("task should be valid")
    }

    #[test]
    fn task_manifest_round_trips_multi_repo_task() {
        let task = fixture_task();

        let encoded = encode_task_manifest(&task).expect("manifest should encode");
        let decoded = decode_task_manifest(&encoded).expect("manifest should decode");

        assert_eq!(decoded, task);
    }
}
