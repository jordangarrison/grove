use std::path::Path;

use crate::domain::Workspace;

#[path = "adapters/metadata.rs"]
mod metadata;
#[path = "adapters/parser.rs"]
mod parser;
#[path = "adapters/workspace.rs"]
mod workspace;

use parser::{parse_branch_activity, parse_worktree_porcelain};
use workspace::build_workspaces;
#[cfg(test)]
use workspace::workspace_name_from_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitAdapterError {
    CommandFailed(String),
    InvalidUtf8(String),
    ParseError(String),
}

impl GitAdapterError {
    pub fn message(&self) -> String {
        match self {
            Self::CommandFailed(message) => format!("git command failed: {message}"),
            Self::InvalidUtf8(message) => format!("git output was not valid UTF-8: {message}"),
            Self::ParseError(message) => format!("git output parse failed: {message}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DiscoveryState {
    Ready,
    Empty,
    Error(String),
}

pub(crate) fn benchmark_discovery_from_synthetic_fixture(
    porcelain_worktrees: &str,
    branch_activity: &str,
    repo_root: &Path,
    repo_name: &str,
) -> Result<Vec<Workspace>, GitAdapterError> {
    let activity_by_branch = parse_branch_activity(branch_activity);
    let parsed_worktrees = parse_worktree_porcelain(porcelain_worktrees)?;
    build_workspaces(&parsed_worktrees, repo_root, repo_name, &activity_by_branch)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::{
        GitAdapterError, build_workspaces, parse_branch_activity, parse_worktree_porcelain,
        workspace_name_from_path,
    };
    use crate::domain::{AgentType, WorkspaceStatus};

    #[derive(Debug)]
    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(label: &str) -> Self {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "grove-adapter-{label}-{}-{timestamp}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("test dir should exist");
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn parse_worktree_porcelain_supports_branch_and_detached_entries() {
        let output = "worktree /repos/grove\nHEAD 123\nbranch refs/heads/main\n\nworktree /repos/grove-feature-a\nHEAD 456\nbranch refs/heads/feature-a\n\nworktree /repos/grove-detached\nHEAD 789\ndetached\n";

        let parsed = parse_worktree_porcelain(output).expect("porcelain should parse");

        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].path, PathBuf::from("/repos/grove"));
        assert_eq!(parsed[0].branch, Some("main".to_string()));
        assert!(!parsed[0].is_detached);

        assert_eq!(parsed[2].path, PathBuf::from("/repos/grove-detached"));
        assert_eq!(parsed[2].branch, None);
        assert!(parsed[2].is_detached);
    }

    #[test]
    fn parse_worktree_porcelain_rejects_metadata_before_worktree() {
        let output = "branch refs/heads/main\nworktree /repos/grove\n";

        let error = parse_worktree_porcelain(output).expect_err("parser should fail");

        assert_eq!(
            error,
            GitAdapterError::ParseError(
                "encountered metadata before any worktree line".to_string()
            )
        );
    }

    #[test]
    fn parse_branch_activity_collects_unix_timestamps() {
        let output = "main 1700000300\nfeature-a 1700000200\ninvalid not-a-number\n";
        let activity = parse_branch_activity(output);

        assert_eq!(activity.get("main"), Some(&1_700_000_300));
        assert_eq!(activity.get("feature-a"), Some(&1_700_000_200));
        assert!(!activity.contains_key("invalid"));
    }

    #[test]
    fn build_workspaces_includes_main_and_unmanaged_worktrees() {
        let temp = TestDir::new("build");
        let main_root = temp.path.join("grove");
        let managed = temp.path.join("grove-feature-a");
        let unmanaged = temp.path.join("grove-unmanaged");

        fs::create_dir_all(&main_root).expect("main should exist");
        fs::create_dir_all(&managed).expect("managed should exist");
        fs::create_dir_all(&unmanaged).expect("unmanaged should exist");
        fs::create_dir_all(managed.join(".grove")).expect(".grove should exist");

        fs::write(managed.join(".grove/base"), "main\n").expect("base marker should exist");

        let parsed = parse_worktree_porcelain(&format!(
                "worktree {}\nHEAD 1\nbranch refs/heads/main\n\nworktree {}\nHEAD 2\nbranch refs/heads/feature-a\n\nworktree {}\nHEAD 3\nbranch refs/heads/unmanaged\n",
                main_root.display(),
                managed.display(),
                unmanaged.display(),
            ))
            .expect("porcelain should parse");

        let activity_by_branch = HashMap::from([
            ("main".to_string(), 1_700_000_400),
            ("feature-a".to_string(), 1_700_000_300),
            ("unmanaged".to_string(), 1_700_000_100),
        ]);

        let workspaces =
            build_workspaces(&parsed, Path::new(&main_root), "grove", &activity_by_branch)
                .expect("workspace build should succeed");

        assert_eq!(workspaces.len(), 3);
        assert_eq!(workspaces[0].name, "grove");
        assert_eq!(workspaces[0].status, WorkspaceStatus::Main);
        assert_eq!(workspaces[0].agent, AgentType::Claude);

        assert_eq!(workspaces[1].name, "feature-a");
        assert_eq!(workspaces[1].agent, AgentType::Claude);
        assert_eq!(workspaces[1].base_branch.as_deref(), Some("main"));

        assert_eq!(workspaces[2].name, "unmanaged");
        assert_eq!(workspaces[2].agent, AgentType::Claude);
        assert_eq!(workspaces[2].base_branch, None);
    }

    #[test]
    fn workspace_name_from_path_strips_repo_prefix_for_non_main_worktrees() {
        let derived = workspace_name_from_path(Path::new("/repos/grove-feature-a"), "grove", false);
        assert_eq!(derived, "feature-a");

        let main = workspace_name_from_path(Path::new("/repos/grove"), "grove", true);
        assert_eq!(main, "grove");
    }
}
