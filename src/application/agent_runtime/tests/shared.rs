pub(super) use std::collections::HashSet;
pub(super) use std::fs;
pub(super) use std::path::PathBuf;
pub(super) use std::process;
pub(super) use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(super) use rusqlite::Connection;

pub(super) use super::super::{
    CaptureChange, CommandExecutionMode, CommandExecutor, LaunchPlan, LaunchRequest,
    LauncherScript, LivePreviewTarget, SessionActivity, agent_supports_in_pane_restart,
    build_launch_plan, build_shell_launch_plan, default_agent_command,
    detect_agent_session_status_in_home, detect_status,
    detect_status_with_session_override_in_home, detect_waiting_prompt, evaluate_capture_change,
    execute_command_with, execute_commands, execute_commands_for_mode, execute_commands_with,
    execute_commands_with_executor, execute_launch_plan, execute_launch_plan_for_mode,
    execute_launch_plan_with, execute_launch_plan_with_executor,
    execute_launch_request_with_result_for_mode, execute_restart_workspace_in_pane_with_result,
    execute_stop_workspace_with_result_for_mode, extract_agent_resume_command,
    git_preview_session_if_ready, git_session_name_for_workspace, kill_workspace_session_command,
    kill_workspace_session_commands, launch_request_for_workspace, live_preview_agent_session,
    live_preview_capture_target_for_tab, live_preview_session_for_tab, poll_interval,
    reconcile_with_sessions, restart_workspace_in_pane_with_io, sanitize_workspace_name,
    session_name_for_workspace, session_name_for_workspace_ref, shell_launch_request_for_workspace,
    shell_session_name_for_workspace, stop_plan, strip_mouse_fragments,
    tmux_capture_error_indicates_missing_session, tmux_launch_error_indicates_duplicate_session,
    trimmed_nonempty, workspace_can_enter_interactive, workspace_can_start_agent,
    workspace_can_stop_agent, workspace_session_for_preview_tab, workspace_should_poll_status,
    workspace_status_session_target, workspace_status_targets_for_polling,
    workspace_status_targets_for_polling_with_live_preview,
};
pub(super) use crate::domain::{AgentType, Workspace, WorkspaceStatus};

pub(super) fn fixture_workspace(name: &str, is_main: bool) -> Workspace {
    Workspace::try_new(
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

pub(super) fn unique_test_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", process::id()));
    fs::create_dir_all(&path).expect("test directory should be created");
    path
}

#[derive(Default)]
pub(super) struct RecordingCommandExecutor {
    pub(super) commands: Vec<Vec<String>>,
    pub(super) launcher_scripts: Vec<(PathBuf, String)>,
}

impl CommandExecutor for RecordingCommandExecutor {
    fn execute(&mut self, command: &[String]) -> std::io::Result<()> {
        self.commands.push(command.to_vec());
        Ok(())
    }

    fn write_launcher_script(&mut self, script: &LauncherScript) -> std::io::Result<()> {
        self.launcher_scripts
            .push((script.path.clone(), script.contents.clone()));
        Ok(())
    }
}
