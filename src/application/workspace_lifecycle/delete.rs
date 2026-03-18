use super::DeleteWorkspaceRequest;

pub(super) fn delete_workspace_with_session_stopper(
    request: DeleteWorkspaceRequest,
    stop_sessions: impl Fn(Option<&str>, Option<&str>, &str),
) -> (Result<(), String>, Vec<String>) {
    let mut warnings = Vec::new();
    if request.kill_tmux_sessions {
        stop_sessions(
            request.task_slug.as_deref(),
            request.project_name.as_deref(),
            request.workspace_name.as_str(),
        );
    }

    let repo_root = match super::requests::resolve_repo_root(request.project_path.as_ref()) {
        Ok(path) => path,
        Err(error) => {
            return (Err(error), warnings);
        }
    };

    if let Err(error) = super::git_ops::run_delete_worktree_git(
        &repo_root,
        &request.workspace_path,
        request.is_missing,
    ) {
        return (Err(error), warnings);
    }

    if request.delete_local_branch
        && let Err(error) = super::git_ops::run_delete_local_branch_git(&repo_root, &request.branch)
    {
        warnings.push(format!("local branch: {error}"));
    }

    if let Some(task_root) = &request.task_root
        && let Err(error) =
            remove_worktree_from_manifest(task_root.as_path(), &request.workspace_path)
    {
        warnings.push(format!("task manifest update: {error}"));
    }

    (Ok(()), warnings)
}

fn remove_worktree_from_manifest(
    task_root: &std::path::Path,
    deleted_workspace_path: &std::path::Path,
) -> Result<(), String> {
    use crate::infrastructure::task_manifest::{decode_task_manifest, encode_task_manifest};

    let manifest_path = task_root.join(".grove/task.toml");
    let raw = std::fs::read_to_string(&manifest_path)
        .map_err(|error| format!("read manifest: {error}"))?;
    let mut task =
        decode_task_manifest(&raw).map_err(|error| format!("decode manifest: {error}"))?;

    let original_count = task.worktrees.len();
    task.worktrees
        .retain(|worktree| worktree.path != deleted_workspace_path);
    if task.worktrees.len() == original_count {
        return Ok(());
    }

    let encoded =
        encode_task_manifest(&task).map_err(|error| format!("encode manifest: {error}"))?;
    std::fs::write(&manifest_path, encoded).map_err(|error| format!("write manifest: {error}"))
}
