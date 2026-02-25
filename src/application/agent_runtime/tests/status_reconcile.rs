use super::*;

#[test]
fn waiting_prompt_checks_tail_lines_only() {
    let output = "approve earlier\nline\nline\nline\nline\nline\nline\nline\nline\n";
    assert_eq!(detect_waiting_prompt(output), None);

    let tail_output = "line\nline\nline\nline\nallow edit? [y/n]\n";
    assert_eq!(
        detect_waiting_prompt(tail_output),
        Some("allow edit? [y/n]".to_string())
    );
}

#[test]
fn waiting_prompt_detects_codex_shortcuts_hint() {
    let output = "result\nresult\n> Implement {feature}\n? for shortcuts\n";
    assert_eq!(
        detect_waiting_prompt(output),
        Some("? for shortcuts".to_string())
    );
}

#[test]
fn waiting_prompt_detects_unicode_prompt_prefix() {
    let output = "Claude Code v2\n› Try \"how does adapters.rs work?\"\n";
    assert_eq!(
        detect_waiting_prompt(output),
        Some("› Try \"how does adapters.rs work?\"".to_string())
    );
    assert_eq!(
        detect_status(output, SessionActivity::Active, false, true, true),
        WorkspaceStatus::Waiting
    );
}

#[test]
fn waiting_prompt_does_not_treat_plain_shell_angle_prompt_as_waiting() {
    let output = "build finished\n> \n";
    assert_eq!(detect_waiting_prompt(output), None);
}

#[test]
fn waiting_prompt_does_not_treat_generic_skills_hint_as_waiting() {
    let output = "Done.\n› Use /skills to list available skills\n";
    assert_eq!(detect_waiting_prompt(output), None);
}

#[test]
fn claude_session_file_marks_waiting_when_last_message_is_assistant() {
    let root = unique_test_dir("grove-claude-session");
    let home = root.join("home");
    let workspace_path = root.join("ws").join("feature-alpha");
    fs::create_dir_all(&home).expect("home directory should exist");
    fs::create_dir_all(&workspace_path).expect("workspace directory should exist");

    let project_dir_name = super::super::claude_project_dir_name(&workspace_path);
    let project_dir = home.join(".claude").join("projects").join(project_dir_name);
    fs::create_dir_all(&project_dir).expect("project directory should exist");

    let session_file = project_dir.join("session-1.jsonl");
    fs::write(
        &session_file,
        "{\"type\":\"system\"}\n{\"type\":\"assistant\"}\n",
    )
    .expect("session file should be written");

    let status = detect_agent_session_status_in_home(
        AgentType::Claude,
        &workspace_path,
        &home,
        Duration::from_secs(0),
    );
    assert_eq!(status, Some(WorkspaceStatus::Waiting));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn codex_session_file_marks_waiting_when_last_message_is_assistant() {
    let root = unique_test_dir("grove-codex-session");
    let home = root.join("home");
    let workspace_path = root.join("ws").join("feature-beta");
    fs::create_dir_all(&home).expect("home directory should exist");
    fs::create_dir_all(&workspace_path).expect("workspace directory should exist");

    let sessions_dir = home.join(".codex").join("sessions").join("2026").join("02");
    fs::create_dir_all(&sessions_dir).expect("sessions directory should exist");
    let session_file = sessions_dir.join("rollout-1.jsonl");
    fs::write(
            &session_file,
            format!(
                "{{\"type\":\"session_meta\",\"payload\":{{\"cwd\":\"{}\"}}}}\n{{\"type\":\"response_item\",\"payload\":{{\"type\":\"message\",\"role\":\"assistant\"}}}}\n",
                workspace_path.display()
            ),
        )
        .expect("session file should be written");

    let status = detect_agent_session_status_in_home(
        AgentType::Codex,
        &workspace_path,
        &home,
        Duration::from_secs(0),
    );
    assert_eq!(status, Some(WorkspaceStatus::Waiting));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn claude_attention_marker_exists_when_last_message_is_assistant() {
    let root = unique_test_dir("grove-claude-attention-marker");
    let home = root.join("home");
    let workspace_path = root.join("ws").join("feature-alpha");
    fs::create_dir_all(&home).expect("home directory should exist");
    fs::create_dir_all(&workspace_path).expect("workspace directory should exist");

    let project_dir_name = super::super::claude_project_dir_name(&workspace_path);
    let project_dir = home.join(".claude").join("projects").join(project_dir_name);
    fs::create_dir_all(&project_dir).expect("project directory should exist");

    let session_file = project_dir.join("session-1.jsonl");
    fs::write(
        &session_file,
        "{\"type\":\"system\"}\n{\"type\":\"assistant\"}\n",
    )
    .expect("session file should be written");

    let marker =
        super::super::latest_claude_assistant_attention_marker_in_home(&workspace_path, &home);
    assert!(marker.is_some());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn codex_attention_marker_exists_when_last_message_is_assistant() {
    let root = unique_test_dir("grove-codex-attention-marker");
    let home = root.join("home");
    let workspace_path = root.join("ws").join("feature-beta");
    fs::create_dir_all(&home).expect("home directory should exist");
    fs::create_dir_all(&workspace_path).expect("workspace directory should exist");

    let sessions_dir = home.join(".codex").join("sessions").join("2026").join("02");
    fs::create_dir_all(&sessions_dir).expect("sessions directory should exist");
    let session_file = sessions_dir.join("rollout-1.jsonl");
    fs::write(
        &session_file,
        format!(
            "{{\"type\":\"session_meta\",\"payload\":{{\"cwd\":\"{}\"}}}}\n{{\"type\":\"response_item\",\"payload\":{{\"type\":\"message\",\"role\":\"assistant\"}}}}\n",
            workspace_path.display()
        ),
    )
    .expect("session file should be written");

    let marker =
        super::super::latest_codex_assistant_attention_marker_in_home(&workspace_path, &home);
    assert!(marker.is_some());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn opencode_session_file_marks_waiting_when_last_message_is_assistant() {
    let root = unique_test_dir("grove-opencode-session");
    let home = root.join("home");
    let workspace_path = root.join("ws").join("feature-delta");
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
                "session-alpha",
                workspace_path.to_string_lossy().to_string(),
                1_i64,
            ),
        )
        .expect("session row should be inserted");
    connection
        .execute(
            "INSERT INTO message (id, session_id, time_created, time_updated, data) VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                "message-alpha",
                "session-alpha",
                1_i64,
                1_i64,
                "{\"role\":\"assistant\"}",
            ),
        )
        .expect("message row should be inserted");

    let status = detect_agent_session_status_in_home(
        AgentType::OpenCode,
        &workspace_path,
        &home,
        Duration::from_secs(0),
    );
    assert_eq!(status, Some(WorkspaceStatus::Waiting));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn opencode_attention_marker_exists_when_last_message_is_assistant() {
    let root = unique_test_dir("grove-opencode-attention-marker");
    let home = root.join("home");
    let workspace_path = root.join("ws").join("feature-epsilon");
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
                "session-beta",
                workspace_path.to_string_lossy().to_string(),
                1_i64,
            ),
        )
        .expect("session row should be inserted");
    connection
        .execute(
            "INSERT INTO message (id, session_id, time_created, time_updated, data) VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                "message-beta",
                "session-beta",
                1_i64,
                2_i64,
                "{\"role\":\"assistant\"}",
            ),
        )
        .expect("message row should be inserted");

    let marker =
        super::super::latest_opencode_assistant_attention_marker_in_home(&workspace_path, &home);
    assert!(marker.is_some());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn status_override_uses_session_files_for_active_waiting() {
    let root = unique_test_dir("grove-status-override");
    let home = root.join("home");
    let workspace_path = root.join("ws").join("feature-gamma");
    fs::create_dir_all(&home).expect("home directory should exist");
    fs::create_dir_all(&workspace_path).expect("workspace directory should exist");

    let project_dir_name = super::super::claude_project_dir_name(&workspace_path);
    let project_dir = home.join(".claude").join("projects").join(project_dir_name);
    fs::create_dir_all(&project_dir).expect("project directory should exist");
    let session_file = project_dir.join("session-2.jsonl");
    fs::write(&session_file, "{\"type\":\"assistant\"}\n").expect("session file should be written");

    let status = detect_status_with_session_override_in_home(super::super::StatusOverrideContext {
        output: "plain output",
        session_activity: SessionActivity::Active,
        is_main: false,
        has_live_session: true,
        supported_agent: true,
        agent: AgentType::Claude,
        workspace_path: &workspace_path,
        home_dir: Some(&home),
        activity_threshold: Duration::from_secs(0),
    });
    assert_eq!(status, WorkspaceStatus::Waiting);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn status_resolution_uses_recent_tail_and_waiting_prompt_before_error() {
    assert_eq!(
        detect_status("panic: bad", SessionActivity::Active, false, true, true),
        WorkspaceStatus::Error
    );
    assert_eq!(
        detect_status(
            "task completed successfully",
            SessionActivity::Active,
            false,
            true,
            true
        ),
        WorkspaceStatus::Done
    );
    assert_eq!(
        detect_status("thinking...", SessionActivity::Active, false, true, true),
        WorkspaceStatus::Thinking
    );
    assert_eq!(
        detect_status(
            "allow edit? [y/n]",
            SessionActivity::Active,
            false,
            true,
            true
        ),
        WorkspaceStatus::Waiting
    );
    assert_eq!(
        detect_status("", SessionActivity::Active, false, true, true),
        WorkspaceStatus::Active
    );
    assert_eq!(
        detect_status("", SessionActivity::Idle, false, false, true),
        WorkspaceStatus::Idle
    );
    assert_eq!(
        detect_status("", SessionActivity::Active, false, true, false),
        WorkspaceStatus::Unsupported
    );

    assert_eq!(
        detect_status(
            "warning: failed to login mcp\nline\nline\n> Implement {feature}\n? for shortcuts\n",
            SessionActivity::Active,
            false,
            true,
            true
        ),
        WorkspaceStatus::Waiting
    );
    assert_eq!(
        detect_status(
            "Do you want to continue?",
            SessionActivity::Active,
            false,
            true,
            true
        ),
        WorkspaceStatus::Waiting
    );
    assert_eq!(
        detect_status("", SessionActivity::Active, true, true, true),
        WorkspaceStatus::Active
    );
    assert_eq!(
        detect_status("", SessionActivity::Idle, true, false, true),
        WorkspaceStatus::Main
    );
    assert_eq!(
        detect_status(
            "task output\n• Done.\n› Use /skills to list available skills\n",
            SessionActivity::Active,
            false,
            true,
            true
        ),
        WorkspaceStatus::Done
    );
}

#[test]
fn status_resolution_detects_done_marker_beyond_last_twelve_lines() {
    let mut lines = vec!["header".to_string(), "• Done.".to_string()];
    lines.extend((0..20).map(|index| format!("detail line {index}")));
    let output = lines.join("\n");
    assert_eq!(
        detect_status(&output, SessionActivity::Active, false, true, true),
        WorkspaceStatus::Done
    );
}

#[test]
fn status_resolution_ignores_old_non_tail_errors() {
    let mut lines = vec!["failed: transient startup warning".to_string()];
    lines.extend((0..70).map(|index| format!("line {index}")));
    let output = lines.join("\n");
    assert_eq!(
        detect_status(&output, SessionActivity::Active, false, true, true),
        WorkspaceStatus::Active
    );
}

#[test]
fn status_resolution_requires_unclosed_thinking_tags() {
    assert_eq!(
        detect_status(
            "<thinking>\nworking\n</thinking>",
            SessionActivity::Active,
            false,
            true,
            true
        ),
        WorkspaceStatus::Active
    );
    assert_eq!(
        detect_status(
            "<thinking>\nworking\n",
            SessionActivity::Active,
            false,
            true,
            true
        ),
        WorkspaceStatus::Thinking
    );
    assert_eq!(
        detect_status(
            "<internal_monologue>\nworking\n",
            SessionActivity::Active,
            false,
            true,
            true
        ),
        WorkspaceStatus::Thinking
    );
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

    let result = reconcile_with_sessions(&workspaces, &running_sessions, &previously_running);
    assert_eq!(result.workspaces[0].status, WorkspaceStatus::Active);
    assert_eq!(result.workspaces[1].status, WorkspaceStatus::Active);
    assert_eq!(result.workspaces[2].status, WorkspaceStatus::Idle);
    assert!(result.workspaces[2].is_orphaned);
    assert_eq!(
        result.orphaned_sessions,
        vec!["grove-ws-zombie".to_string()]
    );
}

#[test]
fn poll_intervals_follow_preview_and_interactive_rules() {
    assert_eq!(
        poll_interval(
            WorkspaceStatus::Active,
            true,
            false,
            true,
            Duration::from_millis(100),
            true
        ),
        Duration::from_millis(50)
    );
    assert_eq!(
        poll_interval(
            WorkspaceStatus::Active,
            true,
            false,
            true,
            Duration::from_secs(5),
            true
        ),
        Duration::from_millis(200)
    );
    assert_eq!(
        poll_interval(
            WorkspaceStatus::Active,
            true,
            false,
            true,
            Duration::from_secs(15),
            false
        ),
        Duration::from_millis(500)
    );
    assert_eq!(
        poll_interval(
            WorkspaceStatus::Active,
            false,
            false,
            false,
            Duration::from_secs(30),
            true
        ),
        Duration::from_secs(10)
    );
    assert_eq!(
        poll_interval(
            WorkspaceStatus::Done,
            true,
            false,
            false,
            Duration::from_secs(30),
            false
        ),
        Duration::from_secs(20)
    );
}
