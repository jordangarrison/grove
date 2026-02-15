use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::agent_runtime::session_name_for_workspace_in_project;
use crate::domain::Workspace;

pub fn recover_working_directory(current_dir: &Path, repo_root: &Path) -> PathBuf {
    if current_dir.exists() {
        return current_dir.to_path_buf();
    }

    repo_root.to_path_buf()
}

pub fn missing_workspace_paths(workspaces: &[Workspace]) -> Vec<PathBuf> {
    let mut missing: Vec<PathBuf> = workspaces
        .iter()
        .filter(|workspace| !workspace.is_main && !workspace.path.exists())
        .map(|workspace| workspace.path.clone())
        .collect();
    missing.sort();
    missing.dedup();
    missing
}

pub fn orphaned_sessions(
    running_sessions: &HashSet<String>,
    workspaces: &[Workspace],
) -> Vec<String> {
    let expected_sessions: HashSet<String> = workspaces
        .iter()
        .map(|workspace| {
            session_name_for_workspace_in_project(
                workspace.project_name.as_deref(),
                &workspace.name,
            )
        })
        .collect();

    let mut orphaned: Vec<String> = running_sessions
        .iter()
        .filter(|session| !expected_sessions.contains(*session))
        .cloned()
        .collect();
    orphaned.sort();
    orphaned
}

pub fn bump_generation(generations: &mut HashMap<String, u64>, workspace_name: &str) -> u64 {
    let entry = generations.entry(workspace_name.to_string()).or_insert(0);
    *entry = entry.saturating_add(1);
    *entry
}

pub fn drop_missing_generations(generations: &mut HashMap<String, u64>, workspaces: &[Workspace]) {
    let active_names: HashSet<String> = workspaces
        .iter()
        .map(|workspace| workspace.name.clone())
        .collect();
    generations.retain(|name, _| active_names.contains(name));
}

#[cfg(test)]
mod tests;
