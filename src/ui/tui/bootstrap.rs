use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::adapters::{
    BootstrapData, CommandGitAdapter, CommandMultiplexerAdapter, CommandSystemAdapter,
    DiscoveryState, MultiplexerAdapter, bootstrap_data,
};
use crate::config::{GroveConfig, MultiplexerKind, ProjectConfig};
use crate::mouse::parse_sidebar_ratio;

use super::{
    CommandTmuxInput, CommandZellijInput, DEFAULT_SIDEBAR_WIDTH_PCT, SIDEBAR_RATIO_FILENAME,
    TmuxInput, WORKSPACE_LAUNCH_PROMPT_FILENAME,
};

pub(super) fn default_sidebar_ratio_path() -> PathBuf {
    match std::env::current_dir() {
        Ok(cwd) => cwd.join(SIDEBAR_RATIO_FILENAME),
        Err(_) => PathBuf::from(SIDEBAR_RATIO_FILENAME),
    }
}

pub(super) fn load_sidebar_ratio(path: &Path) -> u16 {
    let Ok(raw) = fs::read_to_string(path) else {
        return DEFAULT_SIDEBAR_WIDTH_PCT;
    };

    parse_sidebar_ratio(&raw).unwrap_or(DEFAULT_SIDEBAR_WIDTH_PCT)
}

fn default_config_path() -> PathBuf {
    crate::config::config_path().unwrap_or_else(|| PathBuf::from(".config/grove/config.toml"))
}

fn current_repo_root() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let root = String::from_utf8(output.stdout).ok()?;
    let trimmed = root.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path = PathBuf::from(trimmed);
    path.canonicalize().ok().or(Some(path))
}

pub(super) fn project_display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string())
}

pub(super) fn project_paths_equal(left: &Path, right: &Path) -> bool {
    let left_canonical = left.canonicalize().ok();
    let right_canonical = right.canonicalize().ok();
    match (left_canonical, right_canonical) {
        (Some(left), Some(right)) => left == right,
        _ => left == right,
    }
}

fn ensure_current_repo_project(config: &mut GroveConfig, config_path: &Path) -> Option<String> {
    let repo_root = current_repo_root()?;

    let already_present = config
        .projects
        .iter()
        .any(|project| project_paths_equal(&project.path, &repo_root));
    if already_present {
        return None;
    }

    config.projects.push(ProjectConfig {
        name: project_display_name(&repo_root),
        path: repo_root,
    });
    crate::config::save_to_path(config_path, config).err()
}

pub(super) fn load_runtime_config() -> (GroveConfig, PathBuf, Option<String>) {
    let (mut config, config_path, load_error) = match crate::config::load() {
        Ok(loaded) => (loaded.config, loaded.path, None),
        Err(error) => (GroveConfig::default(), default_config_path(), Some(error)),
    };
    let startup_error = ensure_current_repo_project(&mut config, &config_path);
    let error = match (load_error, startup_error) {
        (Some(load_error), Some(startup_error)) => Some(format!(
            "{load_error}; startup project add failed: {startup_error}"
        )),
        (Some(load_error), None) => Some(load_error),
        (None, Some(startup_error)) => Some(format!("startup project add failed: {startup_error}")),
        (None, None) => None,
    };

    (config, config_path, error)
}

#[derive(Debug, Clone)]
struct StaticMultiplexerAdapter {
    running_sessions: HashSet<String>,
}

impl MultiplexerAdapter for StaticMultiplexerAdapter {
    fn running_sessions(&self) -> HashSet<String> {
        self.running_sessions.clone()
    }
}

pub(super) fn bootstrap_data_for_projects(
    projects: &[ProjectConfig],
    multiplexer: MultiplexerKind,
) -> BootstrapData {
    if projects.is_empty() {
        return BootstrapData {
            repo_name: "grove".to_string(),
            workspaces: Vec::new(),
            discovery_state: DiscoveryState::Empty,
            orphaned_sessions: Vec::new(),
        };
    }

    let live_multiplexer = CommandMultiplexerAdapter { multiplexer };
    let static_multiplexer = StaticMultiplexerAdapter {
        running_sessions: live_multiplexer.running_sessions(),
    };
    let mut workspaces = Vec::new();
    let mut orphaned_sessions = Vec::new();
    let mut errors = Vec::new();
    for project in projects {
        let git = CommandGitAdapter::for_repo(project.path.clone());
        let system = CommandSystemAdapter::for_repo(project.path.clone());
        let bootstrap = bootstrap_data(&git, &static_multiplexer, &system);
        if let DiscoveryState::Error(message) = &bootstrap.discovery_state {
            errors.push(format!("{}: {message}", project.name));
        }

        workspaces.extend(bootstrap.workspaces);
        orphaned_sessions.extend(bootstrap.orphaned_sessions);
    }

    let discovery_state = if !workspaces.is_empty() {
        DiscoveryState::Ready
    } else if !errors.is_empty() {
        DiscoveryState::Error(errors.join("; "))
    } else {
        DiscoveryState::Empty
    };
    let repo_name = if projects.len() == 1 {
        projects[0].name.clone()
    } else {
        format!("{} projects", projects.len())
    };

    BootstrapData {
        repo_name,
        workspaces,
        discovery_state,
        orphaned_sessions,
    }
}

pub(super) fn input_for_multiplexer(multiplexer: MultiplexerKind) -> Box<dyn TmuxInput> {
    match multiplexer {
        MultiplexerKind::Tmux => Box::new(CommandTmuxInput),
        MultiplexerKind::Zellij => Box::new(CommandZellijInput::default()),
    }
}

pub(super) fn read_workspace_launch_prompt(workspace_path: &Path) -> Option<String> {
    let raw = fs::read_to_string(workspace_path.join(WORKSPACE_LAUNCH_PROMPT_FILENAME)).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.to_string())
}

pub(super) fn load_local_branches(repo_root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["branch", "--format=%(refname:short)"])
        .output()
        .map_err(|error| format!("git branch failed: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("git branch failed: {stderr}"));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("git branch output decode failed: {error}"))?;
    Ok(stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

pub(super) fn filter_branches(query: &str, all_branches: &[String]) -> Vec<String> {
    if query.is_empty() {
        return all_branches.to_vec();
    }

    let query_lower = query.to_lowercase();
    all_branches
        .iter()
        .filter(|branch| branch.to_lowercase().contains(&query_lower))
        .cloned()
        .collect()
}
