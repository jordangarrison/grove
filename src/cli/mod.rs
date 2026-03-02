use std::fs;
use std::path::{Path, PathBuf};

use crate::infrastructure::event_log::now_millis;

const DEBUG_RECORD_DIR: &str = ".grove";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct CliArgs {
    pub(crate) print_hello: bool,
    pub(crate) event_log_path: Option<PathBuf>,
    pub(crate) debug_record: bool,
    pub(crate) replay_trace_path: Option<PathBuf>,
    pub(crate) replay_snapshot_path: Option<PathBuf>,
    pub(crate) replay_emit_test_name: Option<String>,
    pub(crate) replay_invariant_only: bool,
    pub(crate) benchmark_scale: bool,
    pub(crate) benchmark_json_output: bool,
    pub(crate) benchmark_baseline_path: Option<PathBuf>,
    pub(crate) benchmark_write_baseline_path: Option<PathBuf>,
    pub(crate) benchmark_warn_regression_pct: Option<u64>,
}

pub(crate) fn parse_cli_args(args: impl IntoIterator<Item = String>) -> std::io::Result<CliArgs> {
    let mut cli = CliArgs::default();
    let mut args = args.into_iter();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--print-hello" => {
                cli.print_hello = true;
            }
            "--event-log" => {
                let Some(path) = args.next() else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "--event-log requires a file path",
                    ));
                };
                cli.event_log_path = Some(PathBuf::from(path));
            }
            "--debug-record" => {
                cli.debug_record = true;
            }
            "replay" => {
                if cli.benchmark_scale {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "replay cannot be combined with benchmark-scale",
                    ));
                }
                let Some(path) = args.next() else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "replay requires a trace path",
                    ));
                };
                cli.replay_trace_path = Some(PathBuf::from(path));
            }
            "benchmark-scale" => {
                if cli.replay_trace_path.is_some() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "benchmark-scale cannot be combined with replay",
                    ));
                }
                cli.benchmark_scale = true;
            }
            "--snapshot" => {
                let Some(path) = args.next() else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "--snapshot requires a file path",
                    ));
                };
                cli.replay_snapshot_path = Some(PathBuf::from(path));
            }
            "--emit-test" => {
                let Some(name) = args.next() else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "--emit-test requires a fixture name",
                    ));
                };
                cli.replay_emit_test_name = Some(name);
            }
            "--invariant-only" => {
                cli.replay_invariant_only = true;
            }
            "--json" => {
                cli.benchmark_json_output = true;
            }
            "--baseline" => {
                let Some(path) = args.next() else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "--baseline requires a file path",
                    ));
                };
                cli.benchmark_baseline_path = Some(PathBuf::from(path));
            }
            "--write-baseline" => {
                let Some(path) = args.next() else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "--write-baseline requires a file path",
                    ));
                };
                cli.benchmark_write_baseline_path = Some(PathBuf::from(path));
            }
            "--warn-regression-pct" => {
                let Some(value) = args.next() else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "--warn-regression-pct requires a positive integer",
                    ));
                };
                let parsed = value.parse::<u64>().map_err(|error| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("--warn-regression-pct must be an integer: {error}"),
                    )
                })?;
                if parsed == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "--warn-regression-pct must be greater than zero",
                    ));
                }
                cli.benchmark_warn_regression_pct = Some(parsed);
            }
            _ => {}
        }
    }

    if cli.replay_trace_path.is_none()
        && (cli.replay_snapshot_path.is_some()
            || cli.replay_emit_test_name.is_some()
            || cli.replay_invariant_only)
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "replay flags require `replay <trace-path>`",
        ));
    }

    if !cli.benchmark_scale
        && (cli.benchmark_json_output
            || cli.benchmark_baseline_path.is_some()
            || cli.benchmark_write_baseline_path.is_some()
            || cli.benchmark_warn_regression_pct.is_some())
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "benchmark flags require `benchmark-scale`",
        ));
    }

    Ok(cli)
}

pub(crate) fn debug_record_path(app_start_ts: u64) -> std::io::Result<PathBuf> {
    let dir = PathBuf::from(DEBUG_RECORD_DIR);
    fs::create_dir_all(&dir)?;

    let mut sequence = 0u32;
    loop {
        let file_name = if sequence == 0 {
            format!("debug-record-{app_start_ts}-{}.jsonl", std::process::id())
        } else {
            format!(
                "debug-record-{app_start_ts}-{}-{sequence}.jsonl",
                std::process::id()
            )
        };
        let path = dir.join(file_name);
        if !path.exists() {
            return Ok(path);
        }
        sequence = sequence.saturating_add(1);
    }
}

pub(crate) fn resolve_event_log_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }

    let grove_dir = Path::new(DEBUG_RECORD_DIR);
    if path.starts_with(grove_dir) {
        return path;
    }

    grove_dir.join(path)
}

pub(crate) fn ensure_event_log_parent_directory(path: &Path) -> std::io::Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    fs::create_dir_all(parent)
}

pub fn run(args: impl IntoIterator<Item = String>) -> std::io::Result<()> {
    let cli = parse_cli_args(args)?;

    if cli.benchmark_scale {
        let options = crate::application::scale_benchmark::ScaleBenchmarkOptions {
            json_output: cli.benchmark_json_output,
            baseline_path: cli.benchmark_baseline_path,
            write_baseline_path: cli.benchmark_write_baseline_path,
            severe_regression_pct: cli.benchmark_warn_regression_pct.unwrap_or(
                crate::application::scale_benchmark::ScaleBenchmarkOptions::default()
                    .severe_regression_pct,
            ),
        };
        return crate::application::scale_benchmark::run_scale_benchmark(options);
    }

    if let Some(trace_path) = cli.replay_trace_path.as_ref() {
        if let Some(name) = cli.replay_emit_test_name.as_deref() {
            let fixture_path = crate::ui::tui::emit_replay_fixture(trace_path, name)?;
            println!("replay fixture written: {}", fixture_path.display());
        }

        let options = crate::ui::tui::ReplayOptions {
            invariant_only: cli.replay_invariant_only,
            snapshot_path: cli.replay_snapshot_path.clone(),
        };
        let outcome = crate::ui::tui::replay_debug_record(trace_path, &options)?;
        println!(
            "replay ok: steps={} states={} frames={}",
            outcome.steps_replayed, outcome.states_compared, outcome.frames_compared
        );
        return Ok(());
    }

    let app_start_ts = now_millis();
    let debug_record_path = if cli.debug_record {
        Some(debug_record_path(app_start_ts)?)
    } else {
        None
    };
    if let Some(path) = debug_record_path.as_ref() {
        eprintln!("grove debug record: {}", path.display());
    }
    let event_log_path = debug_record_path.or(cli.event_log_path.map(resolve_event_log_path));
    if let Some(path) = event_log_path.as_ref() {
        ensure_event_log_parent_directory(path)?;
    }

    if cli.print_hello {
        if let Some(event_log_path) = event_log_path.as_ref() {
            let _ = crate::infrastructure::event_log::FileEventLogger::open(event_log_path)?;
        }
        println!("Hello from grove.");
        return Ok(());
    }

    if cli.debug_record
        && let Some(path) = event_log_path
    {
        return crate::ui::tui::run_with_debug_record(path, app_start_ts);
    }

    crate::ui::tui::run_with_event_log(event_log_path)
}

#[cfg(test)]
mod tests {
    use super::{
        CliArgs, debug_record_path, ensure_event_log_parent_directory, parse_cli_args,
        resolve_event_log_path,
    };
    use std::path::PathBuf;

    #[test]
    fn cli_parser_reads_event_log_and_print_hello() {
        let parsed = parse_cli_args(vec![
            "--event-log".to_string(),
            "/tmp/events.jsonl".to_string(),
            "--print-hello".to_string(),
        ])
        .expect("arguments should parse");

        assert_eq!(
            parsed,
            CliArgs {
                print_hello: true,
                event_log_path: Some(PathBuf::from("/tmp/events.jsonl")),
                debug_record: false,
                replay_trace_path: None,
                replay_snapshot_path: None,
                replay_emit_test_name: None,
                replay_invariant_only: false,
                benchmark_scale: false,
                benchmark_json_output: false,
                benchmark_baseline_path: None,
                benchmark_write_baseline_path: None,
                benchmark_warn_regression_pct: None,
            }
        );
    }

    #[test]
    fn cli_parser_requires_event_log_path() {
        let error = parse_cli_args(vec!["--event-log".to_string()])
            .expect_err("missing event log path should fail");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn cli_parser_reads_debug_record_flag() {
        let parsed =
            parse_cli_args(vec!["--debug-record".to_string()]).expect("debug flag should parse");
        assert_eq!(
            parsed,
            CliArgs {
                print_hello: false,
                event_log_path: None,
                debug_record: true,
                replay_trace_path: None,
                replay_snapshot_path: None,
                replay_emit_test_name: None,
                replay_invariant_only: false,
                benchmark_scale: false,
                benchmark_json_output: false,
                benchmark_baseline_path: None,
                benchmark_write_baseline_path: None,
                benchmark_warn_regression_pct: None,
            }
        );
    }

    #[test]
    fn cli_parser_reads_replay_options() {
        let parsed = parse_cli_args(vec![
            "replay".to_string(),
            "/tmp/debug-record.jsonl".to_string(),
            "--snapshot".to_string(),
            "/tmp/replay-snapshot.json".to_string(),
            "--emit-test".to_string(),
            "flow-a".to_string(),
            "--invariant-only".to_string(),
        ])
        .expect("replay arguments should parse");

        assert_eq!(
            parsed,
            CliArgs {
                print_hello: false,
                event_log_path: None,
                debug_record: false,
                replay_trace_path: Some(PathBuf::from("/tmp/debug-record.jsonl")),
                replay_snapshot_path: Some(PathBuf::from("/tmp/replay-snapshot.json")),
                replay_emit_test_name: Some("flow-a".to_string()),
                replay_invariant_only: true,
                benchmark_scale: false,
                benchmark_json_output: false,
                benchmark_baseline_path: None,
                benchmark_write_baseline_path: None,
                benchmark_warn_regression_pct: None,
            }
        );
    }

    #[test]
    fn cli_parser_rejects_replay_flags_without_replay_subcommand() {
        let error = parse_cli_args(vec!["--snapshot".to_string(), "/tmp/out.json".to_string()])
            .expect_err("replay-only flags without replay should fail");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn cli_parser_reads_benchmark_scale_options() {
        let parsed = parse_cli_args(vec![
            "benchmark-scale".to_string(),
            "--json".to_string(),
            "--baseline".to_string(),
            "/tmp/baseline.json".to_string(),
            "--write-baseline".to_string(),
            "/tmp/new-baseline.json".to_string(),
            "--warn-regression-pct".to_string(),
            "25".to_string(),
        ])
        .expect("benchmark arguments should parse");

        assert_eq!(
            parsed,
            CliArgs {
                print_hello: false,
                event_log_path: None,
                debug_record: false,
                replay_trace_path: None,
                replay_snapshot_path: None,
                replay_emit_test_name: None,
                replay_invariant_only: false,
                benchmark_scale: true,
                benchmark_json_output: true,
                benchmark_baseline_path: Some(PathBuf::from("/tmp/baseline.json")),
                benchmark_write_baseline_path: Some(PathBuf::from("/tmp/new-baseline.json")),
                benchmark_warn_regression_pct: Some(25),
            }
        );
    }

    #[test]
    fn cli_parser_rejects_benchmark_flags_without_benchmark_subcommand() {
        let error = parse_cli_args(vec!["--json".to_string()])
            .expect_err("benchmark flags without benchmark command should fail");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn cli_parser_rejects_benchmark_and_replay_combination() {
        let error = parse_cli_args(vec![
            "benchmark-scale".to_string(),
            "replay".to_string(),
            "/tmp/trace.jsonl".to_string(),
        ])
        .expect_err("benchmark and replay should not combine");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn debug_record_path_uses_grove_directory_and_timestamp_prefix() {
        let app_start_ts = 1_771_023_000_555u64;
        let path = debug_record_path(app_start_ts).expect("path should resolve");
        let path_text = path.to_string_lossy();
        assert!(path_text.contains(".grove/"));
        assert!(path_text.contains(&format!("debug-record-{app_start_ts}")));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn resolve_event_log_path_places_relative_paths_under_grove_directory() {
        assert_eq!(
            resolve_event_log_path(PathBuf::from("events.jsonl")),
            PathBuf::from(".grove/events.jsonl")
        );
    }

    #[test]
    fn resolve_event_log_path_keeps_absolute_paths_unchanged() {
        assert_eq!(
            resolve_event_log_path(PathBuf::from("/tmp/events.jsonl")),
            PathBuf::from("/tmp/events.jsonl")
        );
    }

    #[test]
    fn resolve_event_log_path_keeps_grove_prefixed_relative_paths() {
        assert_eq!(
            resolve_event_log_path(PathBuf::from(".grove/custom/events.jsonl")),
            PathBuf::from(".grove/custom/events.jsonl")
        );
    }

    #[test]
    fn ensure_event_log_parent_directory_creates_missing_directories() {
        let root = std::env::temp_dir().join(format!(
            "grove-main-tests-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock should be after unix epoch")
                .as_nanos()
        ));
        let path = root.join(".grove/nested/events.jsonl");

        ensure_event_log_parent_directory(&path).expect("parent directory should be created");
        assert!(root.join(".grove/nested").exists());

        let _ = std::fs::remove_dir_all(root);
    }
}
