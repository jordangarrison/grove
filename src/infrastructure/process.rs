use std::process::{Command, Output};

const SHELL_INIT_GETCWD_PREFIX: &str = "shell-init: error retrieving current directory: getcwd:";

pub(crate) fn stderr_trimmed(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter(|line| !line.starts_with(SHELL_INIT_GETCWD_PREFIX))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\n")
}

pub(crate) fn stderr_or_status(output: &Output) -> String {
    let stderr = stderr_trimmed(output);
    if !stderr.is_empty() {
        return stderr;
    }

    format!("exit status {}", output.status)
}

pub(crate) fn execute_command(command: &[String]) -> std::io::Result<()> {
    if command.is_empty() {
        return Ok(());
    }

    let output = Command::new(&command[0]).args(&command[1..]).output()?;
    if output.status.success() {
        return Ok(());
    }

    Err(std::io::Error::other(format!(
        "command failed: {}; {}",
        command.join(" "),
        stderr_or_status(&output),
    )))
}

#[cfg(test)]
mod tests {
    use super::{execute_command, stderr_trimmed};
    use std::os::unix::process::ExitStatusExt;
    use std::process::Output;

    #[test]
    fn execute_command_ignores_empty_commands() {
        let result = execute_command(&Vec::new());
        assert!(result.is_ok());
    }

    #[test]
    fn execute_command_reports_exit_status_when_stderr_missing() {
        let command = vec!["sh".to_string(), "-c".to_string(), "exit 7".to_string()];
        let result = execute_command(&command);
        let error_text = result.expect_err("command should fail").to_string();

        assert_eq!(
            error_text,
            "command failed: sh -c exit 7; exit status exit status: 7"
        );
    }

    #[test]
    fn execute_command_reports_stderr_when_available() {
        let command = vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo boom >&2; exit 1".to_string(),
        ];
        let result = execute_command(&command);
        let error_text = result.expect_err("command should fail").to_string();

        assert_eq!(
            error_text,
            "command failed: sh -c echo boom >&2; exit 1; boom"
        );
    }

    #[test]
    fn stderr_trimmed_ignores_shell_init_getcwd_noise() {
        let output = Output {
            status: std::process::ExitStatus::from_raw(256),
            stdout: Vec::new(),
            stderr: b"shell-init: error retrieving current directory: getcwd: cannot access parent directories: No such file or directory\nboom\n".to_vec(),
        };

        assert_eq!(stderr_trimmed(&output), "boom");
    }
}
