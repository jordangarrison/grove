use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::domain::{AgentType, Workspace, WorkspaceStatus};

mod agents;

pub const TMUX_SESSION_PREFIX: &str = "grove-ws-";
const GROVE_LAUNCHER_SCRIPT_PATH: &str = ".grove/start.sh";
const WAITING_PATTERNS: [&str; 9] = [
    "[y/n]",
    "(y/n)",
    "allow edit",
    "allow bash",
    "press enter",
    "continue?",
    "do you want",
    "approve",
    "confirm",
];
const WAITING_TAIL_LINES: usize = 8;
const STATUS_TAIL_LINES: usize = 60;
const SESSION_STATUS_TAIL_BYTES: usize = 256 * 1024;
const SESSION_ACTIVITY_THRESHOLD: Duration = Duration::from_secs(30);
const CODEX_SESSION_LOOKUP_REFRESH_INTERVAL: Duration = Duration::from_secs(30);
const OPENCODE_SESSION_LOOKUP_REFRESH_INTERVAL: Duration = Duration::from_millis(500);
const RESTART_RESUME_SCROLLBACK_LINES: usize = 240;
const RESTART_RESUME_CAPTURE_ATTEMPTS: usize = 30;
#[cfg(test)]
const RESTART_RESUME_RETRY_DELAY: Duration = Duration::from_millis(0);
#[cfg(not(test))]
const RESTART_RESUME_RETRY_DELAY: Duration = Duration::from_millis(120);
const OPENCODE_UNSAFE_PERMISSION_JSON: &str = r#"{"*":"allow"}"#;
const DONE_PATTERNS: [&str; 5] = [
    "task completed",
    "all done",
    "finished",
    "exited with code 0",
    "goodbye",
];
const ERROR_PATTERNS: [&str; 6] = [
    "error:",
    "failed",
    "exited with code 1",
    "panic:",
    "exception:",
    "traceback",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionActivity {
    Idle,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchRequest {
    pub project_name: Option<String>,
    pub workspace_name: String,
    pub workspace_path: PathBuf,
    pub agent: AgentType,
    pub prompt: Option<String>,
    pub pre_launch_command: Option<String>,
    pub skip_permissions: bool,
    pub capture_cols: Option<u16>,
    pub capture_rows: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherScript {
    pub path: PathBuf,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    pub session_name: String,
    pub pane_lookup_cmd: Vec<String>,
    pub pre_launch_cmds: Vec<Vec<String>>,
    pub launch_cmd: Vec<String>,
    pub launcher_script: Option<LauncherScript>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionExecutionResult {
    pub workspace_name: String,
    pub workspace_path: PathBuf,
    pub session_name: String,
    pub result: Result<(), String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellLaunchRequest {
    pub session_name: String,
    pub workspace_path: PathBuf,
    pub command: String,
    pub capture_cols: Option<u16>,
    pub capture_rows: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationResult {
    pub workspaces: Vec<Workspace>,
    pub orphaned_sessions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceStatusTarget {
    pub workspace_name: String,
    pub workspace_path: PathBuf,
    pub session_name: String,
    pub supported_agent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LivePreviewTarget {
    pub session_name: String,
    pub include_escape_sequences: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OutputDigest {
    pub raw_hash: u64,
    pub raw_len: usize,
    pub cleaned_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CaptureChange {
    pub digest: OutputDigest,
    pub changed_raw: bool,
    pub changed_cleaned: bool,
    pub cleaned_output: String,
    pub render_output: String,
}

pub(crate) fn sanitize_workspace_name(name: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;

    for character in name.chars() {
        let mapped = if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
            character
        } else {
            '-'
        };

        if mapped == '-' {
            if !last_dash {
                out.push('-');
            }
            last_dash = true;
            continue;
        }

        out.push(mapped);
        last_dash = false;
    }

    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        return "workspace".to_string();
    }

    trimmed.to_string()
}

#[cfg(test)]
pub fn session_name_for_workspace(workspace_name: &str) -> String {
    session_name_for_workspace_in_project(None, workspace_name)
}

pub fn session_name_for_workspace_ref(workspace: &Workspace) -> String {
    session_name_for_workspace_in_project(workspace.project_name.as_deref(), &workspace.name)
}

pub fn git_session_name_for_workspace(workspace: &Workspace) -> String {
    format!("{}-git", session_name_for_workspace_ref(workspace))
}

pub fn shell_session_name_for_workspace(workspace: &Workspace) -> String {
    format!("{}-shell", session_name_for_workspace_ref(workspace))
}

pub fn workspace_should_poll_status(workspace: &Workspace) -> bool {
    if !workspace.supported_agent {
        return false;
    }

    workspace.status.has_session()
}

pub fn live_preview_agent_session(workspace: Option<&Workspace>) -> Option<String> {
    let workspace = workspace?;
    if !workspace.status.has_session() {
        return None;
    }

    Some(session_name_for_workspace_ref(workspace))
}

pub fn workspace_can_enter_interactive(
    workspace: Option<&Workspace>,
    preview_tab_is_git: bool,
) -> bool {
    if preview_tab_is_git {
        return workspace.is_some();
    }

    live_preview_agent_session(workspace).is_some()
}

pub fn workspace_can_start_agent(workspace: Option<&Workspace>) -> bool {
    let Some(workspace) = workspace else {
        return false;
    };
    if !workspace.supported_agent {
        return false;
    }

    matches!(
        workspace.status,
        WorkspaceStatus::Main
            | WorkspaceStatus::Idle
            | WorkspaceStatus::Done
            | WorkspaceStatus::Error
            | WorkspaceStatus::Unknown
    )
}

pub fn workspace_can_stop_agent(workspace: Option<&Workspace>) -> bool {
    let Some(workspace) = workspace else {
        return false;
    };

    workspace.status.has_session()
}

pub fn launch_request_for_workspace(
    workspace: &Workspace,
    prompt: Option<String>,
    pre_launch_command: Option<String>,
    skip_permissions: bool,
    capture_cols: Option<u16>,
    capture_rows: Option<u16>,
) -> LaunchRequest {
    LaunchRequest {
        project_name: workspace.project_name.clone(),
        workspace_name: workspace.name.clone(),
        workspace_path: workspace.path.clone(),
        agent: workspace.agent,
        prompt,
        pre_launch_command,
        skip_permissions,
        capture_cols,
        capture_rows,
    }
}

pub fn shell_launch_request_for_workspace(
    workspace: &Workspace,
    session_name: String,
    command: String,
    capture_cols: Option<u16>,
    capture_rows: Option<u16>,
) -> ShellLaunchRequest {
    ShellLaunchRequest {
        session_name,
        workspace_path: workspace.path.clone(),
        command,
        capture_cols,
        capture_rows,
    }
}

pub fn workspace_session_for_preview_tab(
    workspace: Option<&Workspace>,
    preview_tab_is_git: bool,
    git_preview_session: Option<&str>,
) -> Option<String> {
    if preview_tab_is_git {
        workspace?;
        return git_preview_session.map(str::to_string);
    }

    live_preview_agent_session(workspace)
}

pub fn git_preview_session_if_ready(
    workspace: Option<&Workspace>,
    ready_sessions: &HashSet<String>,
) -> Option<String> {
    let workspace = workspace?;
    let session_name = git_session_name_for_workspace(workspace);
    if !ready_sessions.contains(&session_name) {
        return None;
    }

    Some(session_name)
}

pub fn live_preview_session_for_tab(
    workspace: Option<&Workspace>,
    preview_tab_is_git: bool,
    ready_sessions: &HashSet<String>,
) -> Option<String> {
    if preview_tab_is_git {
        return git_preview_session_if_ready(workspace, ready_sessions);
    }

    live_preview_agent_session(workspace)
}

pub fn live_preview_capture_target_for_tab(
    workspace: Option<&Workspace>,
    preview_tab_is_git: bool,
    ready_sessions: &HashSet<String>,
) -> Option<LivePreviewTarget> {
    let session_name = live_preview_session_for_tab(workspace, preview_tab_is_git, ready_sessions)?;
    Some(LivePreviewTarget {
        session_name,
        include_escape_sequences: true,
    })
}

pub fn workspace_status_session_target(
    workspace: &Workspace,
    selected_live_session: Option<&str>,
) -> Option<String> {
    if !workspace_should_poll_status(workspace) {
        return None;
    }

    let session_name = session_name_for_workspace_ref(workspace);
    if selected_live_session == Some(session_name.as_str()) {
        return None;
    }

    Some(session_name)
}

pub fn workspace_status_targets_for_polling(
    workspaces: &[Workspace],
    selected_live_session: Option<&str>,
) -> Vec<WorkspaceStatusTarget> {
    workspaces
        .iter()
        .filter_map(|workspace| {
            let session_name = workspace_status_session_target(workspace, selected_live_session)?;
            Some(WorkspaceStatusTarget {
                workspace_name: workspace.name.clone(),
                workspace_path: workspace.path.clone(),
                session_name,
                supported_agent: workspace.supported_agent,
            })
        })
        .collect()
}

pub fn workspace_status_targets_for_polling_with_live_preview(
    workspaces: &[Workspace],
    live_preview: Option<&LivePreviewTarget>,
) -> Vec<WorkspaceStatusTarget> {
    workspace_status_targets_for_polling(
        workspaces,
        live_preview.map(|target| target.session_name.as_str()),
    )
}

pub fn tmux_capture_error_indicates_missing_session(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    lower.contains("can't find pane")
        || lower.contains("can't find session")
        || lower.contains("no server running")
        || lower.contains("no sessions")
        || lower.contains("failed to connect to server")
        || lower.contains("no active session")
        || lower.contains("session not found")
}

pub fn tmux_launch_error_indicates_duplicate_session(error: &str) -> bool {
    error.to_ascii_lowercase().contains("duplicate session")
}

pub fn session_name_for_workspace_in_project(
    project_name: Option<&str>,
    workspace_name: &str,
) -> String {
    if let Some(project_name) = project_name {
        let project = sanitize_workspace_name(project_name);
        return format!(
            "{TMUX_SESSION_PREFIX}{project}-{}",
            sanitize_workspace_name(workspace_name)
        );
    }

    format!(
        "{TMUX_SESSION_PREFIX}{}",
        sanitize_workspace_name(workspace_name)
    )
}

pub fn build_launch_plan(request: &LaunchRequest) -> LaunchPlan {
    let session_name = session_name_for_workspace_in_project(
        request.project_name.as_deref(),
        &request.workspace_name,
    );
    let agent_cmd = build_agent_command(request.agent, request.skip_permissions);
    let pre_launch_command = request
        .pre_launch_command
        .as_deref()
        .and_then(trimmed_nonempty);
    let launch_agent_cmd =
        launch_command_with_pre_launch(&agent_cmd, pre_launch_command.as_deref());
    let mut plan = tmux_launch_plan(request, session_name, launch_agent_cmd);
    if let Some(resize_cmd) = launch_resize_window_command(
        &plan.session_name,
        request.capture_cols,
        request.capture_rows,
    ) {
        plan.pre_launch_cmds.push(resize_cmd);
    }
    plan
}

pub fn build_shell_launch_plan(request: &ShellLaunchRequest) -> LaunchPlan {
    let shared = LaunchRequest {
        project_name: None,
        workspace_name: request.session_name.clone(),
        workspace_path: request.workspace_path.clone(),
        agent: AgentType::Codex,
        prompt: None,
        pre_launch_command: None,
        skip_permissions: false,
        capture_cols: request.capture_cols,
        capture_rows: request.capture_rows,
    };
    let mut plan = tmux_launch_plan(
        &shared,
        request.session_name.clone(),
        request.command.clone(),
    );
    if let Some(resize_cmd) = launch_resize_window_command(
        &plan.session_name,
        request.capture_cols,
        request.capture_rows,
    ) {
        plan.pre_launch_cmds.push(resize_cmd);
    }
    if request.command.trim().is_empty() {
        plan.launch_cmd = Vec::new();
    }
    plan
}

fn tmux_launch_plan(
    request: &LaunchRequest,
    session_name: String,
    launch_agent_cmd: String,
) -> LaunchPlan {
    let session_target = session_name.clone();
    let pre_launch_cmds = vec![
        vec![
            "tmux".to_string(),
            "new-session".to_string(),
            "-d".to_string(),
            "-s".to_string(),
            session_name.clone(),
            "-c".to_string(),
            request.workspace_path.to_string_lossy().to_string(),
        ],
        vec![
            "tmux".to_string(),
            "set-option".to_string(),
            "-t".to_string(),
            session_name.clone(),
            "history-limit".to_string(),
            "10000".to_string(),
        ],
    ];
    let pane_lookup_cmd = vec![
        "tmux".to_string(),
        "list-panes".to_string(),
        "-t".to_string(),
        session_name.clone(),
        "-F".to_string(),
        "#{pane_id}".to_string(),
    ];

    match &request.prompt {
        None => LaunchPlan {
            session_name,
            pane_lookup_cmd,
            pre_launch_cmds,
            launch_cmd: vec![
                "tmux".to_string(),
                "send-keys".to_string(),
                "-t".to_string(),
                session_target,
                launch_agent_cmd,
                "Enter".to_string(),
            ],
            launcher_script: None,
        },
        Some(prompt) => {
            let launcher_path = request.workspace_path.join(GROVE_LAUNCHER_SCRIPT_PATH);
            let launcher_contents =
                build_launcher_script(&launch_agent_cmd, prompt, &launcher_path);
            LaunchPlan {
                session_name,
                pane_lookup_cmd,
                pre_launch_cmds,
                launch_cmd: vec![
                    "tmux".to_string(),
                    "send-keys".to_string(),
                    "-t".to_string(),
                    session_target,
                    format!("bash {}", launcher_path.to_string_lossy()),
                    "Enter".to_string(),
                ],
                launcher_script: Some(LauncherScript {
                    path: launcher_path,
                    contents: launcher_contents,
                }),
            }
        }
    }
}

fn launch_resize_window_command(
    session_name: &str,
    capture_cols: Option<u16>,
    capture_rows: Option<u16>,
) -> Option<Vec<String>> {
    let cols = capture_cols.filter(|value| *value > 0)?;
    let rows = capture_rows.filter(|value| *value > 0)?;
    Some(vec![
        "tmux".to_string(),
        "resize-window".to_string(),
        "-t".to_string(),
        session_name.to_string(),
        "-x".to_string(),
        cols.to_string(),
        "-y".to_string(),
        rows.to_string(),
    ])
}

pub fn stop_plan(session_name: &str) -> Vec<Vec<String>> {
    vec![
        vec![
            "tmux".to_string(),
            "send-keys".to_string(),
            "-t".to_string(),
            session_name.to_string(),
            "C-c".to_string(),
        ],
        vec![
            "tmux".to_string(),
            "kill-session".to_string(),
            "-t".to_string(),
            session_name.to_string(),
        ],
    ]
}

pub fn agent_supports_in_pane_restart(agent: AgentType) -> bool {
    agents::supports_in_pane_restart(agent)
}

enum RestartExitInput {
    Literal(&'static str),
    Named(&'static str),
}

fn restart_exit_input(agent: AgentType) -> Option<RestartExitInput> {
    agents::restart_exit_input(agent)
}

fn restart_exit_plan(session_name: &str, exit_input: RestartExitInput) -> Vec<Vec<String>> {
    match exit_input {
        RestartExitInput::Literal(text) => vec![
            vec![
                "tmux".to_string(),
                "send-keys".to_string(),
                "-l".to_string(),
                "-t".to_string(),
                session_name.to_string(),
                text.to_string(),
            ],
            vec![
                "tmux".to_string(),
                "send-keys".to_string(),
                "-t".to_string(),
                session_name.to_string(),
                "Enter".to_string(),
            ],
        ],
        RestartExitInput::Named(key) => vec![vec![
            "tmux".to_string(),
            "send-keys".to_string(),
            "-t".to_string(),
            session_name.to_string(),
            key.to_string(),
        ]],
    }
}

fn resume_command_with_skip_permissions(
    agent: AgentType,
    command: &str,
    skip_permissions: bool,
) -> String {
    agents::resume_command_with_skip_permissions(agent, command, skip_permissions)
}

fn restart_resume_command(
    session_name: &str,
    agent: AgentType,
    command: &str,
    skip_permissions: bool,
) -> Vec<String> {
    let command = resume_command_with_skip_permissions(agent, command, skip_permissions);
    vec![
        "tmux".to_string(),
        "send-keys".to_string(),
        "-t".to_string(),
        session_name.to_string(),
        command,
        "Enter".to_string(),
    ]
}

pub fn extract_agent_resume_command(agent: AgentType, output: &str) -> Option<String> {
    agents::extract_resume_command(agent, output)
}

pub fn infer_workspace_skip_permissions(agent: AgentType, workspace_path: &Path) -> Option<bool> {
    let home_dir = dirs::home_dir()?;
    if !workspace_path.exists() {
        return None;
    }

    agents::infer_skip_permissions_in_home(agent, workspace_path, &home_dir)
}

fn wait_for_resume_command(
    agent: AgentType,
    session_name: &str,
    capture_output: &mut impl FnMut(&str, usize, bool) -> std::io::Result<String>,
) -> Result<String, String> {
    for attempt in 0..RESTART_RESUME_CAPTURE_ATTEMPTS {
        let output = capture_output(session_name, RESTART_RESUME_SCROLLBACK_LINES, false)
            .map_err(|error| error.to_string())?;
        if let Some(command) = extract_agent_resume_command(agent, output.as_str()) {
            return Ok(command);
        }
        if attempt + 1 < RESTART_RESUME_CAPTURE_ATTEMPTS {
            std::thread::sleep(RESTART_RESUME_RETRY_DELAY);
        }
    }

    Err(format!(
        "resume command not found in tmux output for '{session_name}'"
    ))
}

pub fn restart_workspace_in_pane_with_io(
    workspace: &Workspace,
    skip_permissions: bool,
    mut execute: impl FnMut(&[String]) -> std::io::Result<()>,
    mut capture_output: impl FnMut(&str, usize, bool) -> std::io::Result<String>,
) -> Result<(), String> {
    let home_dir = dirs::home_dir();
    restart_workspace_in_pane_with_io_in_home(
        workspace,
        skip_permissions,
        &mut execute,
        &mut capture_output,
        home_dir.as_deref(),
    )
}

fn restart_workspace_in_pane_with_io_in_home(
    workspace: &Workspace,
    skip_permissions: bool,
    mut execute: impl FnMut(&[String]) -> std::io::Result<()>,
    mut capture_output: impl FnMut(&str, usize, bool) -> std::io::Result<String>,
    home_dir: Option<&Path>,
) -> Result<(), String> {
    let Some(exit_input) = restart_exit_input(workspace.agent) else {
        return Err(format!(
            "in-pane restart unsupported for {}",
            workspace.agent.label()
        ));
    };
    let session_name = session_name_for_workspace_ref(workspace);

    for command in restart_exit_plan(&session_name, exit_input) {
        execute_command_with(command.as_slice(), |command| execute(command))
            .map_err(|error| error.to_string())?;
    }

    let resume_command =
        wait_for_resume_command(workspace.agent, &session_name, &mut capture_output).or_else(
            |error| {
                let Some(home_dir) = home_dir else {
                    return Err(error);
                };
                agents::infer_resume_command_in_home(workspace.agent, &workspace.path, home_dir)
                    .ok_or(error)
            },
        )?;
    let command = restart_resume_command(
        &session_name,
        workspace.agent,
        resume_command.as_str(),
        skip_permissions,
    );
    execute_command_with(command.as_slice(), |command| execute(command))
        .map_err(|error| error.to_string())
}

fn capture_output_with_process(
    target_session: &str,
    scrollback_lines: usize,
    include_escape_sequences: bool,
) -> std::io::Result<String> {
    let mut args = vec![
        "capture-pane".to_string(),
        "-p".to_string(),
        "-N".to_string(),
    ];
    if include_escape_sequences {
        args.push("-e".to_string());
    }
    args.push("-t".to_string());
    args.push(target_session.to_string());
    args.push("-S".to_string());
    args.push(format!("-{scrollback_lines}"));

    let output = std::process::Command::new("tmux").args(args).output()?;
    if !output.status.success() {
        let stderr = crate::infrastructure::process::stderr_or_status(&output);
        return Err(std::io::Error::other(format!(
            "tmux capture-pane failed for '{target_session}': {stderr}"
        )));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| std::io::Error::other(format!("tmux output utf8 decode failed: {error}")))
}

pub fn execute_restart_workspace_in_pane_with_result(
    workspace: &Workspace,
    skip_permissions: bool,
) -> SessionExecutionResult {
    let workspace_name = workspace.name.clone();
    let workspace_path = workspace.path.clone();
    let session_name = session_name_for_workspace_ref(workspace);
    let result = restart_workspace_in_pane_with_io(
        workspace,
        skip_permissions,
        crate::infrastructure::process::execute_command,
        capture_output_with_process,
    );
    SessionExecutionResult {
        workspace_name,
        workspace_path,
        session_name,
        result,
    }
}

#[cfg(test)]
pub fn execute_launch_plan(launch_plan: LaunchPlan) -> std::io::Result<()> {
    let mut executor = ProcessCommandExecutor;
    execute_launch_plan_with_executor(&launch_plan, &mut executor)
}

pub enum CommandExecutionMode<'a> {
    Process,
    Delegating(&'a mut dyn FnMut(&[String]) -> std::io::Result<()>),
}

pub fn execute_launch_request_with_result_for_mode(
    request: &LaunchRequest,
    mode: CommandExecutionMode<'_>,
) -> SessionExecutionResult {
    let workspace_name = request.workspace_name.clone();
    let workspace_path = request.workspace_path.clone();
    let launch_plan = build_launch_plan(request);
    let session_name = launch_plan.session_name.clone();
    let result = execute_launch_plan_for_mode(&launch_plan, mode);
    SessionExecutionResult {
        workspace_name,
        workspace_path,
        session_name,
        result,
    }
}

pub fn execute_shell_launch_request_for_mode(
    request: &ShellLaunchRequest,
    mode: CommandExecutionMode<'_>,
) -> (String, Result<(), String>) {
    let launch_plan = build_shell_launch_plan(request);
    let session_name = launch_plan.session_name.clone();
    let result = execute_launch_plan_for_mode(&launch_plan, mode);
    (session_name, result)
}

pub fn execute_stop_workspace_with_result_for_mode(
    workspace: &Workspace,
    mode: CommandExecutionMode<'_>,
) -> SessionExecutionResult {
    let workspace_name = workspace.name.clone();
    let workspace_path = workspace.path.clone();
    let session_name = session_name_for_workspace_ref(workspace);
    let commands = stop_plan(&session_name);
    let result = execute_commands_for_mode(&commands, mode);
    SessionExecutionResult {
        workspace_name,
        workspace_path,
        session_name,
        result,
    }
}

pub fn execute_launch_plan_for_mode(
    launch_plan: &LaunchPlan,
    mode: CommandExecutionMode<'_>,
) -> Result<(), String> {
    match mode {
        CommandExecutionMode::Process => {
            let mut executor = ProcessCommandExecutor;
            execute_launch_plan_with_executor(launch_plan, &mut executor)
                .map_err(|error| error.to_string())
        }
        CommandExecutionMode::Delegating(execute) => {
            let mut executor = DelegatingCommandExecutor::new(execute)
                .with_script_write_error_prefix("launcher script write failed: ");
            execute_launch_plan_with_executor(launch_plan, &mut executor)
                .map_err(|error| error.to_string())
        }
    }
}

#[cfg(test)]
pub fn execute_commands(commands: &[Vec<String>]) -> std::io::Result<()> {
    let mut executor = ProcessCommandExecutor;
    execute_commands_with_executor(commands, &mut executor)
}

pub fn execute_commands_for_mode(
    commands: &[Vec<String>],
    mode: CommandExecutionMode<'_>,
) -> Result<(), String> {
    match mode {
        CommandExecutionMode::Process => {
            let mut executor = ProcessCommandExecutor;
            execute_commands_with_executor(commands, &mut executor)
                .map_err(|error| error.to_string())
        }
        CommandExecutionMode::Delegating(execute) => {
            let mut executor = DelegatingCommandExecutor::new(execute);
            execute_commands_with_executor(commands, &mut executor)
                .map_err(|error| error.to_string())
        }
    }
}

pub fn execute_command_with(
    command: &[String],
    execute: impl FnOnce(&[String]) -> std::io::Result<()>,
) -> std::io::Result<()> {
    if command.is_empty() {
        return Ok(());
    }

    execute(command)
}

pub fn execute_commands_with(
    commands: &[Vec<String>],
    mut execute: impl FnMut(&[String]) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let mut executor = DelegatingCommandExecutor::new(&mut execute);
    execute_commands_with_executor(commands, &mut executor)
}

pub fn execute_launch_plan_with(
    launch_plan: &LaunchPlan,
    mut execute: impl FnMut(&[String]) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let mut executor = DelegatingCommandExecutor::new(&mut execute)
        .with_script_write_error_prefix("launcher script write failed: ");
    execute_launch_plan_with_executor(launch_plan, &mut executor)
}

pub trait CommandExecutor {
    fn execute(&mut self, command: &[String]) -> std::io::Result<()>;

    fn write_launcher_script(&mut self, script: &LauncherScript) -> std::io::Result<()> {
        write_launcher_script_to_disk(script)
    }
}

pub struct ProcessCommandExecutor;

impl CommandExecutor for ProcessCommandExecutor {
    fn execute(&mut self, command: &[String]) -> std::io::Result<()> {
        crate::infrastructure::process::execute_command(command)
    }
}

pub struct DelegatingCommandExecutor<'a> {
    execute: &'a mut dyn FnMut(&[String]) -> std::io::Result<()>,
    script_write_error_prefix: Option<&'a str>,
}

impl<'a> DelegatingCommandExecutor<'a> {
    pub fn new(execute: &'a mut dyn FnMut(&[String]) -> std::io::Result<()>) -> Self {
        Self {
            execute,
            script_write_error_prefix: None,
        }
    }

    pub fn with_script_write_error_prefix(mut self, prefix: &'a str) -> Self {
        self.script_write_error_prefix = Some(prefix);
        self
    }
}

impl CommandExecutor for DelegatingCommandExecutor<'_> {
    fn execute(&mut self, command: &[String]) -> std::io::Result<()> {
        (self.execute)(command)
    }

    fn write_launcher_script(&mut self, script: &LauncherScript) -> std::io::Result<()> {
        match self.script_write_error_prefix {
            Some(prefix) => write_launcher_script_to_disk(script)
                .map_err(|error| std::io::Error::other(format!("{prefix}{error}"))),
            None => write_launcher_script_to_disk(script),
        }
    }
}

fn write_launcher_script_to_disk(script: &LauncherScript) -> std::io::Result<()> {
    if let Some(parent) = script.path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&script.path, &script.contents)
}

pub fn execute_commands_with_executor(
    commands: &[Vec<String>],
    executor: &mut impl CommandExecutor,
) -> std::io::Result<()> {
    for command in commands {
        execute_command_with(command, |command| executor.execute(command))?;
    }
    Ok(())
}

pub fn execute_launch_plan_with_executor(
    launch_plan: &LaunchPlan,
    executor: &mut impl CommandExecutor,
) -> std::io::Result<()> {
    if let Some(script) = &launch_plan.launcher_script {
        executor.write_launcher_script(script)?;
    }

    execute_commands_with_executor(&launch_plan.pre_launch_cmds, executor)?;
    execute_command_with(&launch_plan.launch_cmd, |command| executor.execute(command))
}

pub fn kill_workspace_session_command(
    project_name: Option<&str>,
    workspace_name: &str,
) -> Vec<String> {
    let session_name = session_name_for_workspace_in_project(project_name, workspace_name);
    kill_tmux_session_command(&session_name)
}

pub fn kill_workspace_session_commands(
    project_name: Option<&str>,
    workspace_name: &str,
) -> Vec<Vec<String>> {
    let session_name = session_name_for_workspace_in_project(project_name, workspace_name);
    vec![
        kill_tmux_session_command(&session_name),
        kill_tmux_session_command(&format!("{session_name}-git")),
        kill_tmux_session_command(&format!("{session_name}-shell")),
    ]
}

fn kill_tmux_session_command(session_name: &str) -> Vec<String> {
    vec![
        "tmux".to_string(),
        "kill-session".to_string(),
        "-t".to_string(),
        session_name.to_string(),
    ]
}

pub(crate) fn build_agent_command(agent: AgentType, skip_permissions: bool) -> String {
    if let Some(command_override) = env_agent_command_override(agent) {
        return command_override;
    }

    default_agent_command(agent, skip_permissions)
}

fn default_agent_command(agent: AgentType, skip_permissions: bool) -> String {
    match (agent, skip_permissions) {
        (AgentType::Claude, true) => "claude --dangerously-skip-permissions".to_string(),
        (AgentType::Claude, false) => "claude".to_string(),
        (AgentType::Codex, true) => "codex --dangerously-bypass-approvals-and-sandbox".to_string(),
        (AgentType::Codex, false) => "codex".to_string(),
        (AgentType::OpenCode, true) => {
            format!("OPENCODE_PERMISSION='{OPENCODE_UNSAFE_PERMISSION_JSON}' opencode")
        }
        (AgentType::OpenCode, false) => "opencode".to_string(),
    }
}

fn env_agent_command_override(agent: AgentType) -> Option<String> {
    let variable = agent.command_override_env_var();
    let override_value = std::env::var(variable).ok()?;
    trimmed_nonempty(&override_value)
}

pub(crate) fn trimmed_nonempty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.to_string())
}

fn launch_command_with_pre_launch(agent_command: &str, pre_launch_command: Option<&str>) -> String {
    match pre_launch_command {
        Some(pre_launch) => format!("{pre_launch} && {agent_command}"),
        None => agent_command.to_string(),
    }
}

pub(crate) fn detect_waiting_prompt(output: &str) -> Option<String> {
    let lines: Vec<&str> = output.lines().collect();
    let start = lines.len().saturating_sub(WAITING_TAIL_LINES);
    let tail_lines = &lines[start..];

    for line in tail_lines {
        let lower = line.to_ascii_lowercase();
        if WAITING_PATTERNS
            .iter()
            .any(|pattern| lower.contains(pattern))
        {
            return Some(line.trim().to_string());
        }
    }

    for line in tail_lines.iter().rev() {
        if line.to_ascii_lowercase().contains("for shortcuts") {
            return Some(line.trim().to_string());
        }
    }

    if let Some(last_non_empty) = tail_lines.iter().rev().find(|line| !line.trim().is_empty()) {
        let trimmed = last_non_empty.trim_start();
        let prefix = trimmed.chars().next()?;
        if matches!(prefix, '›' | '❯' | '»') {
            let without_prefix = trimmed.trim_start_matches(['›', '❯', '»']).trim_start();
            if without_prefix.to_ascii_lowercase().starts_with("try ") {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

pub(crate) fn detect_status(
    output: &str,
    session_activity: SessionActivity,
    is_main: bool,
    has_live_session: bool,
    supported_agent: bool,
) -> WorkspaceStatus {
    if is_main && !has_live_session {
        return WorkspaceStatus::Main;
    }

    if !supported_agent {
        return WorkspaceStatus::Unsupported;
    }

    if !has_live_session {
        return WorkspaceStatus::Idle;
    }

    if detect_waiting_prompt(output).is_some() {
        return WorkspaceStatus::Waiting;
    }

    let lines: Vec<&str> = output.lines().collect();
    let start = lines.len().saturating_sub(STATUS_TAIL_LINES);
    let tail_text = lines[start..].join("\n");
    let tail_lower = tail_text.to_ascii_lowercase();

    if has_unclosed_tag(&tail_lower, "<thinking>", "</thinking>")
        || has_unclosed_tag(&tail_lower, "<internal_monologue>", "</internal_monologue>")
        || tail_lower.contains("thinking...")
        || tail_lower.contains("reasoning about")
    {
        return WorkspaceStatus::Thinking;
    }

    if lines[start..].iter().any(|line| {
        let normalized = line
            .trim()
            .trim_start_matches(['•', '-', '*', '·', '✓', '✔', '☑'])
            .trim()
            .to_ascii_lowercase();
        normalized == "done" || normalized == "done."
    }) {
        return WorkspaceStatus::Done;
    }

    if DONE_PATTERNS
        .iter()
        .any(|pattern| tail_lower.contains(pattern))
    {
        return WorkspaceStatus::Done;
    }

    if ERROR_PATTERNS
        .iter()
        .any(|pattern| tail_lower.contains(pattern))
    {
        return WorkspaceStatus::Error;
    }

    match session_activity {
        SessionActivity::Active => WorkspaceStatus::Active,
        SessionActivity::Idle => WorkspaceStatus::Idle,
    }
}

pub(crate) fn detect_status_with_session_override(
    output: &str,
    session_activity: SessionActivity,
    is_main: bool,
    has_live_session: bool,
    supported_agent: bool,
    agent: AgentType,
    workspace_path: &Path,
) -> WorkspaceStatus {
    let home_dir = dirs::home_dir();
    detect_status_with_session_override_in_home(StatusOverrideContext {
        output,
        session_activity,
        is_main,
        has_live_session,
        supported_agent,
        agent,
        workspace_path,
        home_dir: home_dir.as_deref(),
        activity_threshold: SESSION_ACTIVITY_THRESHOLD,
    })
}

struct StatusOverrideContext<'a> {
    output: &'a str,
    session_activity: SessionActivity,
    is_main: bool,
    has_live_session: bool,
    supported_agent: bool,
    agent: AgentType,
    workspace_path: &'a Path,
    home_dir: Option<&'a Path>,
    activity_threshold: Duration,
}

fn detect_status_with_session_override_in_home(
    context: StatusOverrideContext<'_>,
) -> WorkspaceStatus {
    let detected = detect_status(
        context.output,
        context.session_activity,
        context.is_main,
        context.has_live_session,
        context.supported_agent,
    );
    if !matches!(detected, WorkspaceStatus::Active | WorkspaceStatus::Waiting) {
        return detected;
    }

    let Some(home_dir) = context.home_dir else {
        return detected;
    };
    if !context.workspace_path.exists() {
        return detected;
    }

    detect_agent_session_status_in_home(
        context.agent,
        context.workspace_path,
        home_dir,
        context.activity_threshold,
    )
    .unwrap_or(detected)
}

fn detect_agent_session_status_in_home(
    agent: AgentType,
    workspace_path: &Path,
    home_dir: &Path,
    activity_threshold: Duration,
) -> Option<WorkspaceStatus> {
    agents::detect_session_status_in_home(agent, workspace_path, home_dir, activity_threshold)
}

pub(crate) fn latest_assistant_attention_marker(
    agent: AgentType,
    workspace_path: &Path,
) -> Option<String> {
    let home_dir = dirs::home_dir()?;
    if !workspace_path.exists() {
        return None;
    }

    agents::latest_attention_marker_in_home(agent, workspace_path, &home_dir)
}

#[cfg(test)]
fn infer_claude_skip_permissions_in_home(workspace_path: &Path, home_dir: &Path) -> Option<bool> {
    agents::infer_skip_permissions_in_home(AgentType::Claude, workspace_path, home_dir)
}

#[cfg(test)]
fn infer_codex_skip_permissions_in_home(workspace_path: &Path, home_dir: &Path) -> Option<bool> {
    agents::infer_skip_permissions_in_home(AgentType::Codex, workspace_path, home_dir)
}

#[cfg(test)]
fn codex_session_skip_permissions_mode(path: &Path) -> Option<bool> {
    agents::codex_session_skip_permissions_mode(path)
}

#[cfg(test)]
fn infer_opencode_skip_permissions_in_home(workspace_path: &Path, home_dir: &Path) -> Option<bool> {
    agents::infer_skip_permissions_in_home(AgentType::OpenCode, workspace_path, home_dir)
}

#[cfg(test)]
fn latest_claude_assistant_attention_marker_in_home(
    workspace_path: &Path,
    home_dir: &Path,
) -> Option<String> {
    agents::latest_attention_marker_in_home(AgentType::Claude, workspace_path, home_dir)
}

#[cfg(test)]
fn latest_codex_assistant_attention_marker_in_home(
    workspace_path: &Path,
    home_dir: &Path,
) -> Option<String> {
    agents::latest_attention_marker_in_home(AgentType::Codex, workspace_path, home_dir)
}

#[cfg(test)]
fn latest_opencode_assistant_attention_marker_in_home(
    workspace_path: &Path,
    home_dir: &Path,
) -> Option<String> {
    agents::latest_attention_marker_in_home(AgentType::OpenCode, workspace_path, home_dir)
}

#[cfg(test)]
fn claude_project_dir_name(abs_path: &Path) -> String {
    abs_path
        .to_string_lossy()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn has_unclosed_tag(text: &str, open_tag: &str, close_tag: &str) -> bool {
    let Some(open_index) = text.rfind(open_tag) else {
        return false;
    };

    match text.rfind(close_tag) {
        Some(close_index) => close_index < open_index,
        None => true,
    }
}

pub fn reconcile_with_sessions(
    workspaces: &[Workspace],
    running_sessions: &HashSet<String>,
    previously_running_workspace_names: &HashSet<String>,
) -> ReconciliationResult {
    let mut mapped_workspaces = Vec::with_capacity(workspaces.len());
    let mut matched_sessions = HashSet::new();

    for workspace in workspaces {
        let mut updated = workspace.clone();
        let session_name = session_name_for_workspace_in_project(
            workspace.project_name.as_deref(),
            &workspace.name,
        );
        let has_live_session = running_sessions.contains(&session_name);
        if has_live_session {
            matched_sessions.insert(session_name);
            updated.status = detect_status(
                "",
                SessionActivity::Active,
                workspace.is_main,
                true,
                updated.supported_agent,
            );
            updated.is_orphaned = false;
        } else {
            updated.status = detect_status(
                "",
                SessionActivity::Idle,
                workspace.is_main,
                false,
                updated.supported_agent,
            );
            updated.is_orphaned = if workspace.is_main {
                false
            } else {
                previously_running_workspace_names.contains(&workspace.name)
            };
        }

        mapped_workspaces.push(updated);
    }

    let mut orphaned_sessions: Vec<String> = running_sessions
        .iter()
        .filter(|session_name| !matched_sessions.contains(*session_name))
        .cloned()
        .collect();
    orphaned_sessions.sort();

    ReconciliationResult {
        workspaces: mapped_workspaces,
        orphaned_sessions,
    }
}

pub fn poll_interval(
    status: WorkspaceStatus,
    is_selected: bool,
    is_preview_focused: bool,
    interactive_mode: bool,
    since_last_key: Duration,
    output_changing: bool,
) -> Duration {
    if interactive_mode && is_selected {
        if since_last_key < Duration::from_secs(2) {
            return Duration::from_millis(50);
        }
        if since_last_key < Duration::from_secs(10) {
            return Duration::from_millis(200);
        }
        return Duration::from_millis(500);
    }

    if !is_selected {
        return Duration::from_secs(10);
    }

    if output_changing {
        return Duration::from_millis(200);
    }

    if is_preview_focused {
        return Duration::from_millis(500);
    }

    match status {
        WorkspaceStatus::Active | WorkspaceStatus::Thinking => Duration::from_millis(200),
        WorkspaceStatus::Waiting | WorkspaceStatus::Idle => Duration::from_secs(2),
        WorkspaceStatus::Done | WorkspaceStatus::Error => Duration::from_secs(20),
        WorkspaceStatus::Main | WorkspaceStatus::Unknown | WorkspaceStatus::Unsupported => {
            Duration::from_secs(2)
        }
    }
}

pub(crate) fn evaluate_capture_change(
    previous: Option<&OutputDigest>,
    raw_output: &str,
) -> CaptureChange {
    let render_output = strip_non_sgr_control_sequences(raw_output);
    let cleaned_output = strip_mouse_fragments(&strip_sgr_sequences(&render_output));
    let digest = OutputDigest {
        raw_hash: content_hash(raw_output),
        raw_len: raw_output.len(),
        cleaned_hash: content_hash(&cleaned_output),
    };

    match previous {
        None => CaptureChange {
            digest,
            changed_raw: true,
            changed_cleaned: true,
            cleaned_output,
            render_output,
        },
        Some(previous_digest) => CaptureChange {
            changed_raw: previous_digest.raw_hash != digest.raw_hash
                || previous_digest.raw_len != digest.raw_len,
            changed_cleaned: previous_digest.cleaned_hash != digest.cleaned_hash,
            digest,
            cleaned_output,
            render_output,
        },
    }
}

fn is_safe_text_character(character: char) -> bool {
    matches!(character, '\n' | '\t') || !character.is_control()
}

pub(crate) fn strip_mouse_fragments(input: &str) -> String {
    let mut cleaned = input.to_string();

    for mode in [1000u16, 1002, 1003, 1005, 1006, 1015, 2004] {
        cleaned = cleaned.replace(&format!("\u{1b}[?{mode}h"), "");
        cleaned = cleaned.replace(&format!("\u{1b}[?{mode}l"), "");
        cleaned = cleaned.replace(&format!("[?{mode}h"), "");
        cleaned = cleaned.replace(&format!("[?{mode}l"), "");
    }

    strip_partial_mouse_sequences(&cleaned)
}

fn strip_non_sgr_control_sequences(input: &str) -> String {
    let mut cleaned = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        if character != '\u{1b}' {
            if is_safe_text_character(character) {
                cleaned.push(character);
            }
            continue;
        }

        let Some(next) = chars.next() else {
            break;
        };

        match next {
            '[' => {
                let mut csi = String::from("\u{1b}[");
                if let Some(final_char) = consume_csi_sequence(&mut chars, &mut csi)
                    && final_char == 'm'
                {
                    cleaned.push_str(&csi);
                }
            }
            ']' => consume_osc_sequence(&mut chars),
            'P' | 'X' | '^' | '_' => consume_st_sequence(&mut chars),
            '(' | ')' | '*' | '+' | '-' | '.' | '/' | '#' => {
                let _ = chars.next();
            }
            _ => {}
        }
    }

    cleaned
}

fn strip_sgr_sequences(input: &str) -> String {
    let mut cleaned = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '\u{1b}' {
            if chars.next_if_eq(&'[').is_some() {
                let mut did_end = false;
                for value in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&value) {
                        did_end = true;
                        break;
                    }
                }
                if did_end {
                    continue;
                }
            }
            continue;
        }

        if is_safe_text_character(character) {
            cleaned.push(character);
        }
    }

    cleaned
}

fn consume_csi_sequence<I>(chars: &mut std::iter::Peekable<I>, buffer: &mut String) -> Option<char>
where
    I: Iterator<Item = char>,
{
    for character in chars.by_ref() {
        buffer.push(character);
        if ('\u{40}'..='\u{7e}').contains(&character) {
            return Some(character);
        }
    }

    None
}

fn consume_osc_sequence<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
{
    while let Some(character) = chars.next() {
        if character == '\u{7}' {
            return;
        }

        if character == '\u{1b}' && chars.next_if_eq(&'\\').is_some() {
            return;
        }
    }
}

fn consume_st_sequence<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
{
    while let Some(character) = chars.next() {
        if character == '\u{1b}' && chars.next_if_eq(&'\\').is_some() {
            return;
        }
    }
}

fn strip_partial_mouse_sequences(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0usize;

    while index < bytes.len() {
        if let Some(end) = parse_mouse_fragment_end(bytes, index) {
            index = end;
            continue;
        }

        output.push(bytes[index]);
        index += 1;
    }

    String::from_utf8(output).unwrap_or_default()
}

fn parse_mouse_fragment_end(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) == Some(&b'[') && bytes.get(start.saturating_add(1)) == Some(&b'<') {
        return parse_sgr_mouse_tail(bytes, start.saturating_add(2));
    }
    if matches!(bytes.get(start), Some(b'M' | b'm'))
        && bytes.get(start.saturating_add(1)) == Some(&b'[')
        && bytes.get(start.saturating_add(2)) == Some(&b'<')
    {
        return parse_sgr_mouse_tail(bytes, start.saturating_add(3));
    }

    None
}

fn parse_sgr_mouse_tail(bytes: &[u8], mut index: usize) -> Option<usize> {
    index = consume_ascii_digits(bytes, index)?;

    if bytes.get(index) != Some(&b';') {
        return None;
    }
    index = index.saturating_add(1);
    index = consume_ascii_digits(bytes, index)?;

    if bytes.get(index) != Some(&b';') {
        return None;
    }
    index = index.saturating_add(1);
    index = consume_ascii_digits(bytes, index)?;

    if matches!(bytes.get(index), Some(b'M' | b'm')) {
        index = index.saturating_add(1);
    }

    Some(index)
}

fn consume_ascii_digits(bytes: &[u8], mut start: usize) -> Option<usize> {
    let initial = start;
    while matches!(bytes.get(start), Some(b'0'..=b'9')) {
        start = start.saturating_add(1);
    }

    if start == initial { None } else { Some(start) }
}

fn build_launcher_script(agent_cmd: &str, prompt: &str, launcher_path: &Path) -> String {
    format!(
        "#!/bin/bash\nexport NVM_DIR=\"${{NVM_DIR:-$HOME/.nvm}}\"\n[ -s \"$NVM_DIR/nvm.sh\" ] && source \"$NVM_DIR/nvm.sh\" 2>/dev/null\nif ! command -v node &>/dev/null; then\n  [ -f \"$HOME/.zshrc\" ] && source \"$HOME/.zshrc\" 2>/dev/null\n  [ -f \"$HOME/.bashrc\" ] && source \"$HOME/.bashrc\" 2>/dev/null\nfi\n{agent_cmd} \"$(cat <<'GROVE_PROMPT_EOF'\n{prompt}\nGROVE_PROMPT_EOF\n)\"\nrm -f {}\n",
        launcher_path.to_string_lossy()
    )
}

fn content_hash(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests;
