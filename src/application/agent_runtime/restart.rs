use std::path::Path;

use crate::domain::{AgentType, Workspace};

use super::agents;
use super::execution::execute_command_with;
use super::launch_plan::build_agent_env_command;
use super::sessions::session_name_for_workspace_ref;
use super::{
    RESTART_RESUME_CAPTURE_ATTEMPTS, RESTART_RESUME_ERROR_MAX_CHARS,
    RESTART_RESUME_ERROR_TAIL_LINES, RESTART_RESUME_RETRY_DELAY, RESTART_RESUME_SCROLLBACK_LINES,
    RestartExitInput, SessionExecutionResult,
};

pub fn restart_workspace(
    workspace: &Workspace,
    skip_permissions: bool,
    agent_env: Vec<(String, String)>,
) -> SessionExecutionResult {
    execute_restart_workspace_in_pane_with_result(workspace, skip_permissions, agent_env)
}

pub fn agent_supports_in_pane_restart(agent: AgentType) -> bool {
    agents::supports_in_pane_restart(agent)
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
    let mut last_output_excerpt = "<empty>".to_string();
    for attempt in 0..RESTART_RESUME_CAPTURE_ATTEMPTS {
        let output = capture_output(session_name, RESTART_RESUME_SCROLLBACK_LINES, false)
            .map_err(|error| error.to_string())?;
        last_output_excerpt = restart_capture_excerpt(output.as_str());
        if let Some(command) = extract_agent_resume_command(agent, output.as_str()) {
            return Ok(command);
        }
        if attempt + 1 < RESTART_RESUME_CAPTURE_ATTEMPTS {
            std::thread::sleep(RESTART_RESUME_RETRY_DELAY);
        }
    }

    Err(format!(
        "resume command not found in tmux output for '{session_name}' after {} attempts, last_output='{last_output_excerpt}'",
        RESTART_RESUME_CAPTURE_ATTEMPTS
    ))
}

fn restart_capture_excerpt(output: &str) -> String {
    let mut tail = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split_whitespace().collect::<Vec<&str>>().join(" "))
        .rev()
        .take(RESTART_RESUME_ERROR_TAIL_LINES)
        .collect::<Vec<String>>();
    if tail.is_empty() {
        return "<empty>".to_string();
    }
    tail.reverse();
    let joined = tail.join(" | ");
    truncate_excerpt(joined.as_str(), RESTART_RESUME_ERROR_MAX_CHARS)
}

fn truncate_excerpt(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    if max_chars <= 3 {
        return "...".chars().take(max_chars).collect();
    }

    let trimmed: String = value.chars().take(max_chars - 3).collect();
    format!("{trimmed}...")
}

pub fn restart_workspace_in_pane_with_io(
    workspace: &Workspace,
    skip_permissions: bool,
    agent_env: &[(String, String)],
    mut execute: impl FnMut(&[String]) -> std::io::Result<()>,
    mut capture_output: impl FnMut(&str, usize, bool) -> std::io::Result<String>,
) -> Result<(), String> {
    let home_dir = dirs::home_dir();
    restart_workspace_in_pane_with_io_in_home(
        workspace,
        skip_permissions,
        agent_env,
        &mut execute,
        &mut capture_output,
        home_dir.as_deref(),
    )
}

pub(super) fn restart_workspace_in_pane_with_io_in_home(
    workspace: &Workspace,
    skip_permissions: bool,
    agent_env: &[(String, String)],
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
        execute_command_with(command.as_slice(), |command| execute(command)).map_err(|error| {
            format!("restart exit command failed for '{session_name}': {error}")
        })?;
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
    if let Some(command) = restart_agent_env_command(&session_name, agent_env) {
        execute_command_with(command.as_slice(), |command| execute(command))
            .map_err(|error| format!("restart env apply failed for '{session_name}': {error}"))?;
    }
    let command = restart_resume_command(
        &session_name,
        workspace.agent,
        resume_command.as_str(),
        skip_permissions,
    );
    execute_command_with(command.as_slice(), |command| execute(command))
        .map_err(|error| format!("restart resume command failed for '{session_name}': {error}"))
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

fn restart_agent_env_command(
    session_name: &str,
    agent_env: &[(String, String)],
) -> Option<Vec<String>> {
    let env_command = build_agent_env_command(agent_env)?;
    Some(vec![
        "tmux".to_string(),
        "send-keys".to_string(),
        "-t".to_string(),
        session_name.to_string(),
        env_command,
        "Enter".to_string(),
    ])
}

pub fn execute_restart_workspace_in_pane_with_result(
    workspace: &Workspace,
    skip_permissions: bool,
    agent_env: Vec<(String, String)>,
) -> SessionExecutionResult {
    let workspace_name = workspace.name.clone();
    let workspace_path = workspace.path.clone();
    let session_name = session_name_for_workspace_ref(workspace);
    let result = restart_workspace_in_pane_with_io(
        workspace,
        skip_permissions,
        &agent_env,
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
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;

    use rusqlite::Connection;

    use crate::domain::{AgentType, WorkspaceStatus};

    use super::super::status::{
        codex_session_skip_permissions_mode, infer_claude_skip_permissions_in_home,
        infer_codex_skip_permissions_in_home, infer_opencode_skip_permissions_in_home,
    };
    use super::{
        agent_supports_in_pane_restart, execute_restart_workspace_in_pane_with_result,
        extract_agent_resume_command, restart_workspace_in_pane_with_io,
        restart_workspace_in_pane_with_io_in_home,
    };

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

    fn unique_test_dir(prefix: &str) -> PathBuf {
        use std::process;
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", process::id()));
        fs::create_dir_all(&path).expect("test directory should be created");
        path
    }

    #[test]
    fn extract_agent_resume_command_returns_latest_claude_resume_command() {
        let output = "\
        old hint: claude --resume old-session\n\
        another line\n\
        run this next: claude --resume abc123xyz.\n";
        let resume = extract_agent_resume_command(AgentType::Claude, output);
        assert_eq!(resume.as_deref(), Some("claude --resume abc123xyz"));
    }

    #[test]
    fn extract_agent_resume_command_parses_real_claude_exit_output() {
        let output = "\
        Press Ctrl-C again to exit\n\
        \n\
        Resume this session with:\n\
        claude --resume e610b734-e6b8-4b1f-b42f-f3ddeb817467\n";
        let resume = extract_agent_resume_command(AgentType::Claude, output);
        assert_eq!(
            resume.as_deref(),
            Some("claude --resume e610b734-e6b8-4b1f-b42f-f3ddeb817467")
        );
    }

    #[test]
    fn extract_agent_resume_command_parses_claude_resume_with_flag_before_resume() {
        let output = "\
        Resume this session with:\n\
        claude --dangerously-skip-permissions --resume e610b734-e6b8-4b1f-b42f-f3ddeb817467\n";
        let resume = extract_agent_resume_command(AgentType::Claude, output);
        assert_eq!(
            resume.as_deref(),
            Some("claude --resume e610b734-e6b8-4b1f-b42f-f3ddeb817467")
        );
    }

    #[test]
    fn extract_agent_resume_command_parses_claude_short_resume_flag() {
        let output = "Run this next: claude -r e610b734-e6b8-4b1f-b42f-f3ddeb817467";
        let resume = extract_agent_resume_command(AgentType::Claude, output);
        assert_eq!(
            resume.as_deref(),
            Some("claude --resume e610b734-e6b8-4b1f-b42f-f3ddeb817467")
        );
    }

    #[test]
    fn extract_agent_resume_command_parses_real_codex_exit_output() {
        let output = "\
        To continue this session, run codex resume 019c83c1-26c3-7fb0-bd4d-51bb9d6e7701\n";
        let resume = extract_agent_resume_command(AgentType::Codex, output);
        assert_eq!(
            resume.as_deref(),
            Some("codex resume 019c83c1-26c3-7fb0-bd4d-51bb9d6e7701")
        );
    }

    #[test]
    fn extract_agent_resume_command_normalizes_codex_dash_resume_to_subcommand() {
        let output =
            "To continue this session, run codex --resume 019c83c1-26c3-7fb0-bd4d-51bb9d6e7701\n";
        let resume = extract_agent_resume_command(AgentType::Codex, output);
        assert_eq!(
            resume.as_deref(),
            Some("codex resume 019c83c1-26c3-7fb0-bd4d-51bb9d6e7701")
        );
    }

    #[test]
    fn extract_agent_resume_command_ignores_codex_prose_resume_phrase() {
        let output = "\
        No TODOs.\n\
        codex resume is\n\
        Token usage: total=1011482\n\
        To continue this session, run codex resume 019c92cf-2410-7ec3-a8bd-b203b83a6fba\n";
        let resume = extract_agent_resume_command(AgentType::Codex, output);
        assert_eq!(
            resume.as_deref(),
            Some("codex resume 019c92cf-2410-7ec3-a8bd-b203b83a6fba")
        );
    }

    #[test]
    fn extract_agent_resume_command_ignores_codex_placeholder_id() {
        let output = "\
        codex resume <id>\n\
        Token usage: total=587331\n\
        To continue this session, run codex resume 019c92cf-2410-7ec3-a8bd-b203b83a6fba\n";
        let resume = extract_agent_resume_command(AgentType::Codex, output);
        assert_eq!(
            resume.as_deref(),
            Some("codex resume 019c92cf-2410-7ec3-a8bd-b203b83a6fba")
        );
    }

    #[test]
    fn extract_agent_resume_command_ignores_codex_rust_string_artifact() {
        let output = "\
        codex resume run-1234\".to_string(),\n\
        Token usage: total=587331\n\
        To continue this session, run codex resume 019c92cf-2410-7ec3-a8bd-b203b83a6fba\n";
        let resume = extract_agent_resume_command(AgentType::Codex, output);
        assert_eq!(
            resume.as_deref(),
            Some("codex resume 019c92cf-2410-7ec3-a8bd-b203b83a6fba")
        );
    }

    #[test]
    fn codex_session_skip_permissions_mode_detects_approval_policy() {
        let root = unique_test_dir("grove-codex-skip-mode");
        let session_file = root.join("session.jsonl");
        fs::write(
            &session_file,
            "\
        {\"type\":\"session_meta\",\"payload\":{\"cwd\":\"/tmp/ws\"}}\n\
        {\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"developer\",\"content\":[{\"type\":\"input_text\",\"text\":\"Approval policy is currently never.\"}]}}\n",
        )
        .expect("session file should be written");
        assert_eq!(
            codex_session_skip_permissions_mode(&session_file),
            Some(true)
        );

        fs::write(
            &session_file,
            "\
        {\"type\":\"session_meta\",\"payload\":{\"cwd\":\"/tmp/ws\"}}\n\
        {\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"<approval_policy>on-request</approval_policy>\"}]}}\n",
        )
        .expect("session file should be rewritten");
        assert_eq!(
            codex_session_skip_permissions_mode(&session_file),
            Some(false)
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn infer_codex_skip_permissions_in_home_uses_workspace_session_file() {
        let root = unique_test_dir("grove-codex-skip-infer");
        let home = root.join("home");
        let workspace_path = root.join("ws").join("feature-gamma");
        fs::create_dir_all(&home).expect("home directory should exist");
        fs::create_dir_all(&workspace_path).expect("workspace directory should exist");

        let sessions_dir = home
            .join(".codex")
            .join("sessions")
            .join("2026")
            .join("02")
            .join("25");
        fs::create_dir_all(&sessions_dir).expect("sessions directory should exist");

        let older = sessions_dir.join("older.jsonl");
        fs::write(
            &older,
            format!(
                "{{\"type\":\"session_meta\",\"payload\":{{\"cwd\":\"{}\"}}}}\n\
        {{\"type\":\"response_item\",\"payload\":{{\"type\":\"message\",\"role\":\"developer\",\"content\":[{{\"type\":\"input_text\",\"text\":\"<approval_policy>on-request</approval_policy>\"}}]}}}}\n",
                workspace_path.display()
            ),
        )
        .expect("older session should be written");

        let newer = sessions_dir.join("newer.jsonl");
        fs::write(
            &newer,
            format!(
                "{{\"type\":\"session_meta\",\"payload\":{{\"cwd\":\"{}\"}}}}\n\
        {{\"type\":\"response_item\",\"payload\":{{\"type\":\"message\",\"role\":\"developer\",\"content\":[{{\"type\":\"input_text\",\"text\":\"Approval policy is currently never.\"}}]}}}}\n",
                workspace_path.display()
            ),
        )
        .expect("newer session should be written");

        std::thread::sleep(Duration::from_millis(10));
        fs::write(
            &newer,
            format!(
                "{{\"type\":\"session_meta\",\"payload\":{{\"cwd\":\"{}\"}}}}\n\
        {{\"type\":\"response_item\",\"payload\":{{\"type\":\"message\",\"role\":\"developer\",\"content\":[{{\"type\":\"input_text\",\"text\":\"Approval policy is currently never.\"}}]}}}}\n",
                workspace_path.display()
            ),
        )
        .expect("newer session rewrite should refresh mtime");

        assert_eq!(
            infer_codex_skip_permissions_in_home(&workspace_path, &home),
            Some(true)
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn infer_claude_skip_permissions_in_home_uses_project_session_file() {
        let root = unique_test_dir("grove-claude-skip-infer");
        let home = root.join("home");
        let workspace_path = root.join("ws").join("feature-zeta");
        fs::create_dir_all(&home).expect("home directory should exist");
        fs::create_dir_all(&workspace_path).expect("workspace directory should exist");

        let project_dir_name = super::super::status::claude_project_dir_name(&workspace_path);
        let project_dir = home.join(".claude").join("projects").join(project_dir_name);
        fs::create_dir_all(&project_dir).expect("project directory should exist");
        let session_file = project_dir.join("session-1.jsonl");

        fs::write(
            &session_file,
            "{\"type\":\"user\",\"message\":\"<approval_policy>never</approval_policy>\"}\n",
        )
        .expect("session file should be written");
        assert_eq!(
            infer_claude_skip_permissions_in_home(&workspace_path, &home),
            Some(true)
        );

        fs::write(
            &session_file,
            "{\"type\":\"user\",\"message\":\"<approval_policy>on-request</approval_policy>\"}\n",
        )
        .expect("session file should be rewritten");
        assert_eq!(
            infer_claude_skip_permissions_in_home(&workspace_path, &home),
            Some(false)
        );

        fs::write(
            &session_file,
            "{\"type\":\"user\",\"permissionMode\":\"bypassPermissions\"}\n",
        )
        .expect("session file should be rewritten");
        assert_eq!(
            infer_claude_skip_permissions_in_home(&workspace_path, &home),
            Some(true)
        );

        fs::write(
            &session_file,
            "{\"type\":\"user\",\"permissionMode\":\"default\"}\n",
        )
        .expect("session file should be rewritten");
        assert_eq!(
            infer_claude_skip_permissions_in_home(&workspace_path, &home),
            Some(false)
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn infer_opencode_skip_permissions_in_home_uses_message_data() {
        let root = unique_test_dir("grove-opencode-skip-infer");
        let home = root.join("home");
        let workspace_path = root.join("ws").join("feature-eta");
        fs::create_dir_all(&home).expect("home directory should exist");
        fs::create_dir_all(&workspace_path).expect("workspace directory should exist");

        let data_dir = home.join(".local").join("share").join("opencode");
        fs::create_dir_all(&data_dir).expect("opencode data directory should exist");
        let database_path = data_dir.join("opencode.db");
        let connection = Connection::open(&database_path).expect("database should open");
        connection
            .execute_batch(
                "CREATE TABLE session (id TEXT PRIMARY KEY, directory TEXT NOT NULL, time_updated INTEGER NOT NULL);
                     CREATE TABLE message (
                       id TEXT PRIMARY KEY,
                       session_id TEXT NOT NULL,
                       time_created INTEGER NOT NULL,
                       time_updated INTEGER NOT NULL,
                       data TEXT NOT NULL
                     );",
            )
            .expect("schema should be created");
        connection
            .execute(
                "INSERT INTO session (id, directory, time_updated) VALUES (?1, ?2, ?3)",
                (
                    "session-skip",
                    workspace_path.to_string_lossy().to_string(),
                    1_i64,
                ),
            )
            .expect("session row should be inserted");
        connection
            .execute(
                "INSERT INTO message (id, session_id, time_created, time_updated, data) VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    "message-skip",
                    "session-skip",
                    1_i64,
                    1_i64,
                    "{\"role\":\"user\",\"text\":\"Approval policy is currently never.\"}",
                ),
            )
            .expect("message row should be inserted");

        assert_eq!(
            infer_opencode_skip_permissions_in_home(&workspace_path, &home),
            Some(true)
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn extract_agent_resume_command_parses_real_opencode_banner_output() {
        let output = "opencode -s ses_36d243142ffeYteys2MXS86Nnt";
        let resume = extract_agent_resume_command(AgentType::OpenCode, output);
        assert_eq!(
            resume.as_deref(),
            Some("opencode -s ses_36d243142ffeYteys2MXS86Nnt")
        );
    }

    #[test]
    fn extract_agent_resume_command_parses_opencode_continue_output() {
        let output = "To continue this session, run opencode --continue";
        let resume = extract_agent_resume_command(AgentType::OpenCode, output);
        assert_eq!(resume.as_deref(), Some("opencode --continue"));
    }

    #[test]
    fn extract_agent_resume_command_ignores_unrelated_output() {
        let output = "claude --resume abc123xyz";
        assert!(extract_agent_resume_command(AgentType::OpenCode, output).is_none());
    }

    #[test]
    fn restart_workspace_in_pane_with_io_sends_exit_and_resume_commands() {
        let workspace = fixture_workspace("feature-a", false);
        let mut commands = Vec::new();
        let mut captures = vec![
            "still shutting down".to_string(),
            "resume with: claude --resume run-1234".to_string(),
        ];

        let result = restart_workspace_in_pane_with_io(
            &workspace,
            false,
            &[],
            |command| {
                commands.push(command.to_vec());
                Ok(())
            },
            |_session_name, _scrollback_lines, _include_escape_sequences| {
                if captures.is_empty() {
                    return Ok(String::new());
                }
                Ok(captures.remove(0))
            },
        );

        assert!(result.is_ok());
        assert_eq!(
            commands,
            vec![
                vec![
                    "tmux".to_string(),
                    "send-keys".to_string(),
                    "-l".to_string(),
                    "-t".to_string(),
                    "grove-ws-feature-a".to_string(),
                    "/exit".to_string(),
                ],
                vec![
                    "tmux".to_string(),
                    "send-keys".to_string(),
                    "-t".to_string(),
                    "grove-ws-feature-a".to_string(),
                    "Enter".to_string(),
                ],
                vec![
                    "tmux".to_string(),
                    "send-keys".to_string(),
                    "-t".to_string(),
                    "grove-ws-feature-a".to_string(),
                    "claude --resume run-1234".to_string(),
                    "Enter".to_string(),
                ],
            ]
        );
    }

    #[test]
    fn restart_workspace_in_pane_with_io_returns_error_when_resume_missing() {
        let workspace = fixture_workspace("feature-a", false);
        let mut captures = vec![
            "still shutting down".to_string(),
            "no resume command".to_string(),
        ];

        let result = restart_workspace_in_pane_with_io(
            &workspace,
            false,
            &[],
            |_command| Ok(()),
            |_session_name, _scrollback_lines, _include_escape_sequences| {
                if captures.is_empty() {
                    return Ok(String::new());
                }
                Ok(captures.remove(0))
            },
        );

        let error = result.expect_err("missing resume command should fail");
        assert!(error.contains("resume command not found"));
    }

    #[test]
    fn restart_workspace_in_pane_with_io_resume_missing_error_includes_capture_excerpt() {
        let workspace = fixture_workspace("feature-a", false);
        let mut call_count = 0_u8;

        let result = restart_workspace_in_pane_with_io(
            &workspace,
            false,
            &[],
            |_command| Ok(()),
            |_session_name, _scrollback_lines, _include_escape_sequences| {
                call_count = call_count.saturating_add(1);
                if call_count == 1 {
                    return Ok("still shutting down".to_string());
                }
                Ok(
                    "\n\n  no resume command in output  \n  Claude exited successfully  "
                        .to_string(),
                )
            },
        );

        let error = result.expect_err("missing resume command should fail");
        assert!(error.contains("resume command not found"));
        assert!(error.contains("last_output='"));
        assert!(error.contains("no resume command in output"));
        assert!(error.contains("Claude exited successfully"));
    }

    #[test]
    fn restart_workspace_in_pane_with_io_sends_ctrl_c_for_opencode() {
        let mut workspace = fixture_workspace("feature-a", false);
        workspace.agent = AgentType::OpenCode;
        let mut commands = Vec::new();
        let mut captures = vec!["opencode -s ses_36d243142ffeYteys2MXS86Nnt".to_string()];

        let result = restart_workspace_in_pane_with_io(
            &workspace,
            false,
            &[],
            |command| {
                commands.push(command.to_vec());
                Ok(())
            },
            |_session_name, _scrollback_lines, _include_escape_sequences| {
                if captures.is_empty() {
                    return Ok(String::new());
                }
                Ok(captures.remove(0))
            },
        );

        assert!(result.is_ok());
        assert_eq!(
            commands,
            vec![
                vec![
                    "tmux".to_string(),
                    "send-keys".to_string(),
                    "-t".to_string(),
                    "grove-ws-feature-a".to_string(),
                    "C-c".to_string(),
                ],
                vec![
                    "tmux".to_string(),
                    "send-keys".to_string(),
                    "-t".to_string(),
                    "grove-ws-feature-a".to_string(),
                    "opencode -s ses_36d243142ffeYteys2MXS86Nnt".to_string(),
                    "Enter".to_string(),
                ],
            ]
        );
    }

    #[test]
    fn restart_workspace_in_pane_with_io_uses_opencode_db_resume_when_output_missing() {
        let root = unique_test_dir("grove-opencode-restart-resume-fallback");
        let home = root.join("home");
        let workspace_path = root.join("ws").join("feature-opencode");
        fs::create_dir_all(&home).expect("home directory should exist");
        fs::create_dir_all(&workspace_path).expect("workspace directory should exist");

        let data_dir = home.join(".local").join("share").join("opencode");
        fs::create_dir_all(&data_dir).expect("opencode data directory should exist");
        let database_path = data_dir.join("opencode.db");
        let connection = Connection::open(&database_path).expect("database should open");
        connection
            .execute_batch(
                "CREATE TABLE session (id TEXT PRIMARY KEY, directory TEXT NOT NULL, time_updated INTEGER NOT NULL);",
            )
            .expect("schema should be created");
        connection
            .execute(
                "INSERT INTO session (id, directory, time_updated) VALUES (?1, ?2, ?3)",
                (
                    "session-fallback",
                    workspace_path.to_string_lossy().to_string(),
                    1_i64,
                ),
            )
            .expect("session row should be inserted");

        let mut workspace = fixture_workspace("feature-a", false);
        workspace.agent = AgentType::OpenCode;
        workspace.path = workspace_path.clone();
        let mut commands = Vec::new();

        let result = restart_workspace_in_pane_with_io_in_home(
            &workspace,
            false,
            &[],
            |command| {
                commands.push(command.to_vec());
                Ok(())
            },
            |_session_name, _scrollback_lines, _include_escape_sequences| {
                Ok("no resume command in output".to_string())
            },
            Some(home.as_path()),
        );

        assert!(result.is_ok());
        assert_eq!(
            commands[1],
            vec![
                "tmux".to_string(),
                "send-keys".to_string(),
                "-t".to_string(),
                "grove-ws-feature-a".to_string(),
                "opencode -s session-fallback".to_string(),
                "Enter".to_string(),
            ]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn restart_workspace_in_pane_with_io_sends_ctrl_c_for_codex() {
        let mut workspace = fixture_workspace("feature-a", false);
        workspace.agent = AgentType::Codex;
        let mut commands = Vec::new();
        let mut captures = vec!["run codex resume run-1234".to_string()];

        let result = restart_workspace_in_pane_with_io(
            &workspace,
            false,
            &[],
            |command| {
                commands.push(command.to_vec());
                Ok(())
            },
            |_session_name, _scrollback_lines, _include_escape_sequences| {
                if captures.is_empty() {
                    return Ok(String::new());
                }
                Ok(captures.remove(0))
            },
        );

        assert!(result.is_ok());
        assert_eq!(
            commands,
            vec![
                vec![
                    "tmux".to_string(),
                    "send-keys".to_string(),
                    "-t".to_string(),
                    "grove-ws-feature-a".to_string(),
                    "C-c".to_string(),
                ],
                vec![
                    "tmux".to_string(),
                    "send-keys".to_string(),
                    "-t".to_string(),
                    "grove-ws-feature-a".to_string(),
                    "codex resume run-1234".to_string(),
                    "Enter".to_string(),
                ],
            ]
        );
    }

    #[test]
    fn restart_workspace_in_pane_with_io_applies_agent_env_before_resume() {
        let mut workspace = fixture_workspace("feature-a", false);
        workspace.agent = AgentType::Codex;
        let mut commands = Vec::new();
        let mut captures = vec!["run codex resume run-1234".to_string()];

        let result = restart_workspace_in_pane_with_io(
            &workspace,
            false,
            &[
                ("FOO".to_string(), "bar".to_string()),
                ("BAR".to_string(), "baz".to_string()),
            ],
            |command| {
                commands.push(command.to_vec());
                Ok(())
            },
            |_session_name, _scrollback_lines, _include_escape_sequences| {
                if captures.is_empty() {
                    return Ok(String::new());
                }
                Ok(captures.remove(0))
            },
        );

        assert!(result.is_ok());
        assert_eq!(
            commands[1],
            vec![
                "tmux".to_string(),
                "send-keys".to_string(),
                "-t".to_string(),
                "grove-ws-feature-a".to_string(),
                "export FOO='bar' BAR='baz'".to_string(),
                "Enter".to_string(),
            ]
        );
        assert_eq!(
            commands[2],
            vec![
                "tmux".to_string(),
                "send-keys".to_string(),
                "-t".to_string(),
                "grove-ws-feature-a".to_string(),
                "codex resume run-1234".to_string(),
                "Enter".to_string(),
            ]
        );
    }

    #[test]
    fn execute_restart_workspace_in_pane_with_result_returns_workspace_context() {
        let mut workspace = fixture_workspace("feature-a", false);
        workspace.agent = AgentType::OpenCode;

        let result = execute_restart_workspace_in_pane_with_result(&workspace, false, Vec::new());
        assert_eq!(result.workspace_name, "feature-a");
        assert_eq!(
            result.workspace_path,
            PathBuf::from("/repos/grove-feature-a")
        );
        assert_eq!(result.session_name, "grove-ws-feature-a");
        assert!(result.result.is_err());
    }

    #[test]
    fn restart_workspace_in_pane_with_io_adds_skip_permissions_for_codex_resume() {
        let mut workspace = fixture_workspace("feature-a", false);
        workspace.agent = AgentType::Codex;
        let mut commands = Vec::new();
        let mut captures = vec!["run codex resume run-1234".to_string()];

        let result = restart_workspace_in_pane_with_io(
            &workspace,
            true,
            &[],
            |command| {
                commands.push(command.to_vec());
                Ok(())
            },
            |_session_name, _scrollback_lines, _include_escape_sequences| {
                if captures.is_empty() {
                    return Ok(String::new());
                }
                Ok(captures.remove(0))
            },
        );

        assert!(result.is_ok());
        assert_eq!(
            commands[1],
            vec![
                "tmux".to_string(),
                "send-keys".to_string(),
                "-t".to_string(),
                "grove-ws-feature-a".to_string(),
                "codex --dangerously-bypass-approvals-and-sandbox resume run-1234".to_string(),
                "Enter".to_string(),
            ]
        );
    }

    #[test]
    fn restart_workspace_in_pane_with_io_adds_skip_permissions_for_claude_resume() {
        let workspace = fixture_workspace("feature-a", false);
        let mut commands = Vec::new();
        let mut captures = vec!["resume with: claude --resume run-1234".to_string()];

        let result = restart_workspace_in_pane_with_io(
            &workspace,
            true,
            &[],
            |command| {
                commands.push(command.to_vec());
                Ok(())
            },
            |_session_name, _scrollback_lines, _include_escape_sequences| {
                if captures.is_empty() {
                    return Ok(String::new());
                }
                Ok(captures.remove(0))
            },
        );

        assert!(result.is_ok());
        assert_eq!(
            commands[2],
            vec![
                "tmux".to_string(),
                "send-keys".to_string(),
                "-t".to_string(),
                "grove-ws-feature-a".to_string(),
                "claude --dangerously-skip-permissions --resume run-1234".to_string(),
                "Enter".to_string(),
            ]
        );
    }

    #[test]
    fn restart_workspace_in_pane_with_io_adds_skip_permissions_for_codex_dash_resume() {
        let mut workspace = fixture_workspace("feature-a", false);
        workspace.agent = AgentType::Codex;
        let mut commands = Vec::new();
        let mut captures = vec!["run codex --resume run-1234".to_string()];

        let result = restart_workspace_in_pane_with_io(
            &workspace,
            true,
            &[],
            |command| {
                commands.push(command.to_vec());
                Ok(())
            },
            |_session_name, _scrollback_lines, _include_escape_sequences| {
                if captures.is_empty() {
                    return Ok(String::new());
                }
                Ok(captures.remove(0))
            },
        );

        assert!(result.is_ok());
        assert_eq!(
            commands[1],
            vec![
                "tmux".to_string(),
                "send-keys".to_string(),
                "-t".to_string(),
                "grove-ws-feature-a".to_string(),
                "codex --dangerously-bypass-approvals-and-sandbox resume run-1234".to_string(),
                "Enter".to_string(),
            ]
        );
    }

    #[test]
    fn restart_workspace_in_pane_with_io_adds_skip_permissions_for_opencode_resume() {
        let mut workspace = fixture_workspace("feature-a", false);
        workspace.agent = AgentType::OpenCode;
        let mut commands = Vec::new();
        let mut captures = vec!["opencode -s ses_36d243142ffeYteys2MXS86Nnt".to_string()];

        let result = restart_workspace_in_pane_with_io(
            &workspace,
            true,
            &[],
            |command| {
                commands.push(command.to_vec());
                Ok(())
            },
            |_session_name, _scrollback_lines, _include_escape_sequences| {
                if captures.is_empty() {
                    return Ok(String::new());
                }
                Ok(captures.remove(0))
            },
        );

        assert!(result.is_ok());
        assert_eq!(
            commands[1],
            vec![
                "tmux".to_string(),
                "send-keys".to_string(),
                "-t".to_string(),
                "grove-ws-feature-a".to_string(),
                "OPENCODE_PERMISSION='{\"*\":\"allow\"}' opencode -s ses_36d243142ffeYteys2MXS86Nnt"
                    .to_string(),
                "Enter".to_string(),
            ]
        );
    }

    #[test]
    fn agent_supports_in_pane_restart_is_enabled_for_all_agents() {
        assert!(agent_supports_in_pane_restart(AgentType::Claude));
        assert!(agent_supports_in_pane_restart(AgentType::Codex));
        assert!(agent_supports_in_pane_restart(AgentType::OpenCode));
    }
}
