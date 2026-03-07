use std::path::Path;
use std::process::Command;

use crate::application::agent_runtime::{
    kill_workspace_session_commands, kill_workspace_session_commands_for_existing_sessions,
};
use crate::application::workspace_lifecycle::{
    DeleteWorkspaceRequest, MergeWorkspaceRequest, SessionTerminator,
    UpdateWorkspaceFromBaseRequest, WorkspaceLifecycleError,
    delete_workspace_with_terminator as lifecycle_delete_workspace_with_terminator,
    merge_workspace_with_terminator as lifecycle_merge_workspace_with_terminator,
    update_workspace_from_base_with_terminator as lifecycle_update_workspace_from_base_with_terminator,
    workspace_lifecycle_error_message as lifecycle_workspace_lifecycle_error_message,
    write_workspace_base_marker as lifecycle_write_workspace_base_marker,
};
use crate::infrastructure::process::{execute_command, stderr_trimmed};

#[derive(Debug, Clone, Copy, Default)]
struct RuntimeSessionTerminator;

impl SessionTerminator for RuntimeSessionTerminator {
    fn stop_workspace_sessions(
        &self,
        task_slug: Option<&str>,
        project_name: Option<&str>,
        workspace_name: &str,
    ) {
        let commands = match list_tmux_session_names() {
            Ok(existing_sessions) => {
                if existing_sessions.is_empty() {
                    return;
                }
                kill_workspace_session_commands_for_existing_sessions(
                    task_slug,
                    project_name,
                    workspace_name,
                    existing_sessions.as_slice(),
                )
            }
            Err(_) => kill_workspace_session_commands(task_slug, project_name, workspace_name),
        };

        for command in commands {
            if command.is_empty() {
                continue;
            }
            let _ = execute_command(&command);
        }
    }
}

fn list_tmux_session_names() -> Result<Vec<String>, String> {
    let output = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output()
        .map_err(|error| format!("tmux list-sessions failed: {error}"))?;
    if !output.status.success() {
        let stderr = stderr_trimmed(&output);
        if stderr.contains("no server running") {
            return Ok(Vec::new());
        }
        return Err(format!("tmux list-sessions failed: {stderr}"));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("tmux output invalid UTF-8: {error}"))?;
    Ok(stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

pub fn delete_workspace(request: DeleteWorkspaceRequest) -> (Result<(), String>, Vec<String>) {
    lifecycle_delete_workspace_with_terminator(request, &RuntimeSessionTerminator)
}

pub fn merge_workspace(request: MergeWorkspaceRequest) -> (Result<(), String>, Vec<String>) {
    lifecycle_merge_workspace_with_terminator(request, &RuntimeSessionTerminator)
}

pub fn update_workspace_from_base(
    request: UpdateWorkspaceFromBaseRequest,
) -> (Result<(), String>, Vec<String>) {
    lifecycle_update_workspace_from_base_with_terminator(request, &RuntimeSessionTerminator)
}

pub fn workspace_lifecycle_error_message(error: &WorkspaceLifecycleError) -> String {
    lifecycle_workspace_lifecycle_error_message(error)
}

pub fn write_workspace_base_marker(
    workspace_path: &Path,
    base_branch: &str,
) -> Result<(), WorkspaceLifecycleError> {
    lifecycle_write_workspace_base_marker(workspace_path, base_branch)
}
