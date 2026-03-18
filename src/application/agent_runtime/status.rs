use std::path::Path;
use std::time::Duration;

use crate::domain::{AgentType, WorkspaceStatus};

use super::agents;
use super::{
    SESSION_ACTIVITY_THRESHOLD, STATUS_TAIL_LINES, SessionActivity, WAITING_PATTERNS,
    WAITING_TAIL_LINES,
};

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
        let normalized = normalize_status_line(line);
        normalized == "done" || normalized == "done."
    }) {
        return WorkspaceStatus::Done;
    }

    if lines[start..].iter().any(|line| is_done_line(line)) {
        return WorkspaceStatus::Done;
    }

    if lines[start..].iter().any(|line| is_error_line(line)) {
        return WorkspaceStatus::Error;
    }

    match session_activity {
        SessionActivity::Active => WorkspaceStatus::Active,
        SessionActivity::Idle => WorkspaceStatus::Idle,
    }
}

pub(crate) struct StatusOverrideContext<'a> {
    pub(crate) output: &'a str,
    pub(crate) session_activity: SessionActivity,
    pub(crate) is_main: bool,
    pub(crate) has_live_session: bool,
    pub(crate) supported_agent: bool,
    pub(crate) agent: AgentType,
    pub(crate) workspace_path: &'a Path,
    pub(crate) home_dir: Option<&'a Path>,
    pub(crate) activity_threshold: Duration,
}

pub(crate) fn detect_status_with_session_override_in_home(
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

pub(crate) fn detect_agent_session_status_in_home(
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
pub(super) fn infer_claude_skip_permissions_in_home(
    workspace_path: &Path,
    home_dir: &Path,
) -> Option<bool> {
    agents::infer_skip_permissions_in_home(AgentType::Claude, workspace_path, home_dir)
}

#[cfg(test)]
pub(super) fn infer_codex_skip_permissions_in_home(
    workspace_path: &Path,
    home_dir: &Path,
) -> Option<bool> {
    agents::infer_skip_permissions_in_home(AgentType::Codex, workspace_path, home_dir)
}

#[cfg(test)]
pub(super) fn codex_session_skip_permissions_mode(path: &Path) -> Option<bool> {
    agents::codex_session_skip_permissions_mode(path)
}

#[cfg(test)]
pub(super) fn infer_opencode_skip_permissions_in_home(
    workspace_path: &Path,
    home_dir: &Path,
) -> Option<bool> {
    agents::infer_skip_permissions_in_home(AgentType::OpenCode, workspace_path, home_dir)
}

#[cfg(test)]
pub(super) fn latest_claude_assistant_attention_marker_in_home(
    workspace_path: &Path,
    home_dir: &Path,
) -> Option<String> {
    agents::latest_attention_marker_in_home(AgentType::Claude, workspace_path, home_dir)
}

#[cfg(test)]
pub(super) fn latest_codex_assistant_attention_marker_in_home(
    workspace_path: &Path,
    home_dir: &Path,
) -> Option<String> {
    agents::latest_attention_marker_in_home(AgentType::Codex, workspace_path, home_dir)
}

#[cfg(test)]
pub(super) fn latest_opencode_assistant_attention_marker_in_home(
    workspace_path: &Path,
    home_dir: &Path,
) -> Option<String> {
    agents::latest_attention_marker_in_home(AgentType::OpenCode, workspace_path, home_dir)
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

fn normalize_status_line(line: &str) -> String {
    line.trim()
        .trim_start_matches(['•', '-', '*', '·', '✓', '✔', '☑'])
        .trim()
        .to_ascii_lowercase()
}

fn is_error_line(line: &str) -> bool {
    let normalized = normalize_status_line(line);
    normalized.starts_with("error:")
        || normalized.starts_with("panic:")
        || normalized.starts_with("exception:")
        || normalized.starts_with("traceback")
        || normalized.contains("exited with code 1")
}

fn is_done_line(line: &str) -> bool {
    let normalized = normalize_status_line(line);
    matches!(
        normalized.as_str(),
        "done" | "done." | "finished" | "finished."
    ) || normalized.starts_with("task completed")
        || normalized.starts_with("all done")
        || normalized.starts_with("goodbye")
        || normalized.starts_with("exited with code 0")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::Duration;

    use rusqlite::Connection;

    use crate::domain::{AgentType, WorkspaceStatus};
    use crate::test_support::unique_test_dir;

    use super::super::SessionActivity;
    use super::super::agents::claude_project_dir_name;
    use super::{
        StatusOverrideContext, detect_agent_session_status_in_home, detect_status,
        detect_status_with_session_override_in_home, detect_waiting_prompt,
        latest_claude_assistant_attention_marker_in_home,
        latest_codex_assistant_attention_marker_in_home,
        latest_opencode_assistant_attention_marker_in_home,
    };

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

        let project_dir_name = claude_project_dir_name(&workspace_path);
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

        let project_dir_name = claude_project_dir_name(&workspace_path);
        let project_dir = home.join(".claude").join("projects").join(project_dir_name);
        fs::create_dir_all(&project_dir).expect("project directory should exist");

        let session_file = project_dir.join("session-1.jsonl");
        fs::write(
            &session_file,
            "{\"type\":\"system\"}\n{\"type\":\"assistant\"}\n",
        )
        .expect("session file should be written");

        let marker = latest_claude_assistant_attention_marker_in_home(&workspace_path, &home);
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

        let marker = latest_codex_assistant_attention_marker_in_home(&workspace_path, &home);
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

        let marker = latest_opencode_assistant_attention_marker_in_home(&workspace_path, &home);
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

        let project_dir_name = claude_project_dir_name(&workspace_path);
        let project_dir = home.join(".claude").join("projects").join(project_dir_name);
        fs::create_dir_all(&project_dir).expect("project directory should exist");
        let session_file = project_dir.join("session-2.jsonl");
        fs::write(&session_file, "{\"type\":\"assistant\"}\n")
            .expect("session file should be written");

        let status = detect_status_with_session_override_in_home(StatusOverrideContext {
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
    fn status_resolution_does_not_treat_inline_finished_text_as_done() {
        assert_eq!(
            detect_status(
                "Based on the planning summary, the risky migration is not finished yet.\nContinuing implementation now.\n",
                SessionActivity::Active,
                false,
                true,
                true
            ),
            WorkspaceStatus::Active
        );
        assert_eq!(
            detect_status(
                "build finished successfully, now running the next step\n",
                SessionActivity::Active,
                false,
                true,
                true
            ),
            WorkspaceStatus::Active
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
    fn status_resolution_ignores_benign_failed_lines_without_error_markers() {
        assert_eq!(
            detect_status(
                "warning: failed to login mcp\nretrying with cached credentials\n",
                SessionActivity::Active,
                false,
                true,
                true
            ),
            WorkspaceStatus::Active
        );
        assert_eq!(
            detect_status(
                "The previous approach failed, trying a different implementation.\n",
                SessionActivity::Active,
                false,
                true,
                true
            ),
            WorkspaceStatus::Active
        );
    }

    #[test]
    fn status_resolution_detects_line_anchored_error_markers() {
        assert_eq!(
            detect_status(
                "error: permission denied\n",
                SessionActivity::Active,
                false,
                true,
                true
            ),
            WorkspaceStatus::Error
        );
        assert_eq!(
            detect_status(
                "Traceback (most recent call last):\n",
                SessionActivity::Active,
                false,
                true,
                true
            ),
            WorkspaceStatus::Error
        );
        assert_eq!(
            detect_status(
                "tool exited with code 1\n",
                SessionActivity::Active,
                false,
                true,
                true
            ),
            WorkspaceStatus::Error
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
}
