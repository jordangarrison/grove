use super::*;

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
        super::super::codex_session_skip_permissions_mode(&session_file),
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
        super::super::codex_session_skip_permissions_mode(&session_file),
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
        super::super::infer_codex_skip_permissions_in_home(&workspace_path, &home),
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

    let project_dir_name = super::super::claude_project_dir_name(&workspace_path);
    let project_dir = home.join(".claude").join("projects").join(project_dir_name);
    fs::create_dir_all(&project_dir).expect("project directory should exist");
    let session_file = project_dir.join("session-1.jsonl");

    fs::write(
        &session_file,
        "{\"type\":\"user\",\"message\":\"<approval_policy>never</approval_policy>\"}\n",
    )
    .expect("session file should be written");
    assert_eq!(
        super::super::infer_claude_skip_permissions_in_home(&workspace_path, &home),
        Some(true)
    );

    fs::write(
        &session_file,
        "{\"type\":\"user\",\"message\":\"<approval_policy>on-request</approval_policy>\"}\n",
    )
    .expect("session file should be rewritten");
    assert_eq!(
        super::super::infer_claude_skip_permissions_in_home(&workspace_path, &home),
        Some(false)
    );

    fs::write(
        &session_file,
        "{\"type\":\"user\",\"permissionMode\":\"bypassPermissions\"}\n",
    )
    .expect("session file should be rewritten");
    assert_eq!(
        super::super::infer_claude_skip_permissions_in_home(&workspace_path, &home),
        Some(true)
    );

    fs::write(
        &session_file,
        "{\"type\":\"user\",\"permissionMode\":\"default\"}\n",
    )
    .expect("session file should be rewritten");
    assert_eq!(
        super::super::infer_claude_skip_permissions_in_home(&workspace_path, &home),
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
        super::super::infer_opencode_skip_permissions_in_home(&workspace_path, &home),
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
            Ok("\n\n  no resume command in output  \n  Claude exited successfully  ".to_string())
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

    let result = super::super::restart_workspace_in_pane_with_io_in_home(
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
fn execute_commands_runs_successful_command_sequence() {
    let commands = vec![
        vec!["sh".to_string(), "-lc".to_string(), "true".to_string()],
        Vec::new(),
    ];
    assert!(execute_commands(&commands).is_ok());
}

#[test]
fn execute_commands_returns_error_for_missing_program() {
    let commands = vec![vec![
        "grove-this-command-does-not-exist".to_string(),
        "arg".to_string(),
    ]];
    assert!(execute_commands(&commands).is_err());
}

#[test]
fn execute_commands_for_mode_process_returns_string_errors() {
    let commands = vec![vec![
        "grove-this-command-does-not-exist".to_string(),
        "arg".to_string(),
    ]];
    let result = execute_commands_for_mode(&commands, CommandExecutionMode::Process);
    let error_text = result.expect_err("missing program should error");

    assert!(!error_text.is_empty());
}

#[test]
fn execute_launch_request_with_result_for_mode_includes_workspace_context() {
    let request = LaunchRequest {
        project_name: Some("project.one".to_string()),
        workspace_name: "auth-flow".to_string(),
        workspace_path: PathBuf::from("/repos/project.one/worktrees/auth-flow"),
        agent: AgentType::Claude,
        prompt: None,
        pre_launch_command: None,
        skip_permissions: false,
        agent_env: Vec::new(),
        capture_cols: Some(120),
        capture_rows: Some(40),
    };
    let result = execute_launch_request_with_result_for_mode(
        &request,
        CommandExecutionMode::Delegating(&mut |_command| {
            Err(std::io::Error::other("synthetic execution failure"))
        }),
    );

    assert_eq!(result.workspace_name, "auth-flow");
    assert_eq!(
        result.workspace_path,
        PathBuf::from("/repos/project.one/worktrees/auth-flow")
    );
    assert_eq!(result.session_name, "grove-ws-project-one-auth-flow");
    assert!(result.result.is_err());
}

#[test]
fn execute_stop_workspace_with_result_for_mode_includes_workspace_context() {
    let workspace = fixture_workspace("feature/auth.v2", false).with_project_context(
        "project.one".to_string(),
        PathBuf::from("/repos/project.one"),
    );
    let mut commands = Vec::new();
    let result = execute_stop_workspace_with_result_for_mode(
        &workspace,
        CommandExecutionMode::Delegating(&mut |command| {
            commands.push(command.to_vec());
            Ok(())
        }),
    );

    assert_eq!(result.workspace_name, "feature/auth.v2");
    assert_eq!(
        result.workspace_path,
        PathBuf::from("/repos/grove-feature/auth.v2")
    );
    assert_eq!(result.session_name, "grove-ws-project-one-feature-auth-v2");
    assert!(result.result.is_ok());
    assert_eq!(
        commands,
        vec![
            vec![
                "tmux".to_string(),
                "send-keys".to_string(),
                "-t".to_string(),
                "grove-ws-project-one-feature-auth-v2".to_string(),
                "C-c".to_string(),
            ],
            vec![
                "tmux".to_string(),
                "kill-session".to_string(),
                "-t".to_string(),
                "grove-ws-project-one-feature-auth-v2".to_string(),
            ],
        ]
    );
}

#[test]
fn execute_launch_plan_writes_launcher_script_and_executes_commands() {
    let temp_dir = unique_test_dir("execute-launch-plan");
    let script_path = temp_dir.join(".grove/start.sh");
    let launch_plan = LaunchPlan {
        session_name: "grove-ws-test".to_string(),
        pane_lookup_cmd: Vec::new(),
        pre_launch_cmds: vec![vec![
            "sh".to_string(),
            "-lc".to_string(),
            "true".to_string(),
        ]],
        launch_cmd: vec!["sh".to_string(), "-lc".to_string(), "true".to_string()],
        launcher_script: Some(LauncherScript {
            path: script_path.clone(),
            contents: "#!/usr/bin/env bash\necho hi\n".to_string(),
        }),
    };

    let result = execute_launch_plan(launch_plan);
    assert!(result.is_ok());
    assert_eq!(
        fs::read_to_string(script_path).expect("launcher script should be written"),
        "#!/usr/bin/env bash\necho hi\n"
    );
}

#[test]
fn execute_commands_with_uses_supplied_executor() {
    let commands = vec![
        vec!["echo".to_string(), "first".to_string()],
        vec!["echo".to_string(), "second".to_string()],
    ];
    let mut observed = Vec::new();

    let result = execute_commands_with(&commands, |command| {
        observed.push(command.join(" "));
        Ok(())
    });

    assert!(result.is_ok());
    assert_eq!(observed, vec!["echo first", "echo second"]);
}

#[test]
fn execute_commands_with_executor_skips_empty_commands() {
    let commands = vec![
        Vec::new(),
        vec!["echo".to_string(), "ran".to_string()],
        Vec::new(),
    ];
    let mut executor = RecordingCommandExecutor::default();

    let result = execute_commands_with_executor(&commands, &mut executor);

    assert!(result.is_ok());
    assert_eq!(
        executor.commands,
        vec![vec!["echo".to_string(), "ran".to_string()]]
    );
}

#[test]
fn execute_launch_plan_with_executor_runs_prelaunch_then_launch() {
    let launch_plan = LaunchPlan {
        session_name: "grove-ws-test".to_string(),
        pane_lookup_cmd: Vec::new(),
        pre_launch_cmds: vec![
            vec!["echo".to_string(), "one".to_string()],
            vec!["echo".to_string(), "two".to_string()],
        ],
        launch_cmd: vec!["echo".to_string(), "launch".to_string()],
        launcher_script: Some(LauncherScript {
            path: PathBuf::from("/tmp/.grove/start.sh"),
            contents: "#!/usr/bin/env bash\necho hi\n".to_string(),
        }),
    };
    let mut executor = RecordingCommandExecutor::default();

    let result = execute_launch_plan_with_executor(&launch_plan, &mut executor);

    assert!(result.is_ok());
    assert_eq!(
        executor.commands,
        vec![
            vec!["echo".to_string(), "one".to_string()],
            vec!["echo".to_string(), "two".to_string()],
            vec!["echo".to_string(), "launch".to_string()],
        ]
    );
    assert_eq!(executor.launcher_scripts.len(), 1);
}

#[test]
fn execute_command_with_skips_empty_commands() {
    let mut executed = false;

    let result = execute_command_with(&Vec::new(), |_command| {
        executed = true;
        Ok(())
    });

    assert!(result.is_ok());
    assert!(!executed);
}

#[test]
fn execute_command_with_invokes_executor_for_non_empty_commands() {
    let command = vec!["echo".to_string(), "ok".to_string()];
    let mut observed = String::new();

    let result = execute_command_with(&command, |command| {
        observed = command.join(" ");
        Ok(())
    });

    assert!(result.is_ok());
    assert_eq!(observed, "echo ok");
}

#[test]
fn execute_launch_plan_with_prefixes_script_write_errors() {
    let temp_dir = unique_test_dir("execute-launch-plan-sync");
    let blocked_path = temp_dir.join("blocked");
    fs::write(&blocked_path, "not a directory").expect("blocked path should be writable");
    let launch_plan = LaunchPlan {
        session_name: "grove-ws-test".to_string(),
        pane_lookup_cmd: Vec::new(),
        pre_launch_cmds: Vec::new(),
        launch_cmd: vec!["sh".to_string(), "-lc".to_string(), "true".to_string()],
        launcher_script: Some(LauncherScript {
            path: blocked_path.join(".grove/start.sh"),
            contents: "#!/usr/bin/env bash\necho hi\n".to_string(),
        }),
    };

    let result = execute_launch_plan_with(&launch_plan, |_command| Ok(()));
    let error_text = result.expect_err("script write should fail").to_string();

    assert!(error_text.starts_with("launcher script write failed: "));
}

#[test]
fn execute_launch_plan_for_mode_delegating_prefixes_script_write_errors() {
    let temp_dir = unique_test_dir("execute-launch-plan-sync-mode");
    let blocked_path = temp_dir.join("blocked");
    fs::write(&blocked_path, "not a directory").expect("blocked path should be writable");
    let launch_plan = LaunchPlan {
        session_name: "grove-ws-test".to_string(),
        pane_lookup_cmd: Vec::new(),
        pre_launch_cmds: Vec::new(),
        launch_cmd: vec!["sh".to_string(), "-lc".to_string(), "true".to_string()],
        launcher_script: Some(LauncherScript {
            path: blocked_path.join(".grove/start.sh"),
            contents: "#!/usr/bin/env bash\necho hi\n".to_string(),
        }),
    };

    let result = execute_launch_plan_for_mode(
        &launch_plan,
        CommandExecutionMode::Delegating(&mut |_command| Ok(())),
    );
    let error_text = result.expect_err("script write should fail");

    assert!(error_text.starts_with("launcher script write failed: "));
}

#[test]
fn execute_launch_plan_keeps_unprefixed_script_write_errors() {
    let temp_dir = unique_test_dir("execute-launch-plan");
    let blocked_path = temp_dir.join("blocked");
    fs::write(&blocked_path, "not a directory").expect("blocked path should be writable");
    let launch_plan = LaunchPlan {
        session_name: "grove-ws-test".to_string(),
        pane_lookup_cmd: Vec::new(),
        pre_launch_cmds: Vec::new(),
        launch_cmd: vec!["sh".to_string(), "-lc".to_string(), "true".to_string()],
        launcher_script: Some(LauncherScript {
            path: blocked_path.join(".grove/start.sh"),
            contents: "#!/usr/bin/env bash\necho hi\n".to_string(),
        }),
    };

    let result = execute_launch_plan(launch_plan);
    let error_text = result.expect_err("script write should fail").to_string();

    assert!(!error_text.starts_with("launcher script write failed: "));
}

#[test]
fn kill_workspace_session_command_uses_project_scoped_tmux_session_name() {
    assert_eq!(
        kill_workspace_session_command(Some("project.one"), "feature/auth.v2"),
        vec![
            "tmux".to_string(),
            "kill-session".to_string(),
            "-t".to_string(),
            "grove-ws-project-one-feature-auth-v2".to_string(),
        ]
    );
}

#[test]
fn kill_workspace_session_commands_include_agent_git_and_shell_sessions() {
    assert_eq!(
        kill_workspace_session_commands(Some("project.one"), "feature/auth.v2"),
        vec![
            vec![
                "tmux".to_string(),
                "kill-session".to_string(),
                "-t".to_string(),
                "grove-ws-project-one-feature-auth-v2".to_string(),
            ],
            vec![
                "tmux".to_string(),
                "kill-session".to_string(),
                "-t".to_string(),
                "grove-ws-project-one-feature-auth-v2-git".to_string(),
            ],
            vec![
                "tmux".to_string(),
                "kill-session".to_string(),
                "-t".to_string(),
                "grove-ws-project-one-feature-auth-v2-shell".to_string(),
            ],
        ]
    );
}

#[test]
fn launch_plan_with_pre_launch_command_runs_before_agent() {
    let request = LaunchRequest {
        project_name: None,
        workspace_name: "auth-flow".to_string(),
        workspace_path: PathBuf::from("/repos/grove-auth-flow"),
        agent: AgentType::Claude,
        prompt: None,
        pre_launch_command: Some("direnv allow".to_string()),
        skip_permissions: true,
        agent_env: Vec::new(),
        capture_cols: None,
        capture_rows: None,
    };

    let plan = build_launch_plan(&request);
    assert_eq!(
        plan.launch_cmd,
        vec![
            "tmux",
            "send-keys",
            "-t",
            "grove-ws-auth-flow",
            "direnv allow && claude --dangerously-skip-permissions",
            "Enter"
        ]
    );
}
