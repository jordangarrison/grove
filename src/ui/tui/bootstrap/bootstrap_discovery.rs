use std::collections::HashSet;
use std::process::Command;

use crate::application::task_discovery::{
    TaskBootstrapData, TaskDiscoveryState,
    bootstrap_task_data_for_root_with_sessions as discover_task_bootstrap_for_root,
};
use crate::application::task_lifecycle::materialize_base_task_manifest_for_project_in_root;
use crate::infrastructure::config::ProjectConfig;
use std::path::Path;

pub(super) fn bootstrap_task_data_for_root(
    tasks_root: &Path,
    projects: &[ProjectConfig],
) -> TaskBootstrapData {
    let running_sessions = running_task_sessions();
    let bootstrap = discover_task_bootstrap_for_root(tasks_root, &running_sessions);
    if matches!(bootstrap.discovery_state, TaskDiscoveryState::Error(_)) {
        return bootstrap;
    }

    let mut known_tasks = bootstrap.tasks.clone();
    let mut created_manifest = false;
    for project in projects {
        if let Ok(Some(created)) =
            materialize_base_task_manifest_for_project_in_root(tasks_root, project, &known_tasks)
        {
            known_tasks.push(created.task);
            created_manifest = true;
        }
    }

    if !created_manifest {
        return bootstrap;
    }

    discover_task_bootstrap_for_root(tasks_root, &running_sessions)
}

fn running_task_sessions() -> HashSet<String> {
    let output = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output();

    match output {
        Ok(output) if output.status.success() => String::from_utf8(output.stdout)
            .map(|content| {
                content
                    .lines()
                    .filter(|name| name.starts_with("grove-task-") || name.starts_with("grove-wt-"))
                    .map(ToOwned::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        _ => HashSet::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::bootstrap_task_data_for_root;
    use crate::infrastructure::config::ProjectConfig;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
                "grove-bootstrap-discovery-{label}-{}-{timestamp}",
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
    fn bootstrap_task_data_ignores_configured_repo_without_manifest() {
        let temp = TestDir::new("no-manifest");
        let tasks_root = temp.path.join("tasks");
        fs::create_dir_all(&tasks_root).expect("tasks root should exist");

        let bootstrap = bootstrap_task_data_for_root(tasks_root.as_path(), &[]);

        assert!(bootstrap.tasks.is_empty());
        assert_eq!(
            bootstrap.discovery_state,
            crate::application::task_discovery::TaskDiscoveryState::Empty
        );
    }

    #[test]
    fn bootstrap_task_data_keeps_manifest_tasks_only() {
        let temp = TestDir::new("manifest-only");
        let tasks_root = temp.path.join("tasks");
        let task_dir = tasks_root.join("feature-a").join(".grove");
        let feature_path = temp.path.join("worktrees").join("feature-a");
        fs::create_dir_all(&task_dir).expect("task dir should exist");
        fs::create_dir_all(&feature_path).expect("feature path should exist");
        fs::write(
            task_dir.join("task.toml"),
            format!(
                "name = \"feature-a\"\nslug = \"feature-a\"\nroot_path = \"{}\"\nbranch = \"feature-a\"\n\n[[worktrees]]\nrepository_name = \"web-monorepo\"\nrepository_path = \"{}\"\npath = \"{}\"\nbranch = \"feature-a\"\nagent = \"codex\"\nstatus = \"idle\"\nis_orphaned = false\nsupported_agent = true\npull_requests = []\n",
                tasks_root.join("feature-a").display(),
                temp.path.join("repos").join("web-monorepo").display(),
                feature_path.display(),
            ),
        )
        .expect("task manifest should write");

        let bootstrap = bootstrap_task_data_for_root(tasks_root.as_path(), &[]);

        assert_eq!(bootstrap.tasks.len(), 1);
        assert_eq!(bootstrap.tasks[0].slug, "feature-a");
    }

    #[test]
    fn bootstrap_task_data_materializes_missing_base_task_manifest_for_project() {
        let temp = TestDir::new("migrate-project");
        let tasks_root = temp.path.join("tasks");
        let repo_root = temp.path.join("repos").join("mcp");
        fs::create_dir_all(&repo_root).expect("repo root should exist");
        let init_output = Command::new("git")
            .current_dir(&repo_root)
            .args(["init", "--initial-branch=main"])
            .output()
            .expect("git init should run");
        assert!(init_output.status.success(), "git init should succeed");
        let user_name_output = Command::new("git")
            .current_dir(&repo_root)
            .args(["config", "user.name", "Grove Tests"])
            .output()
            .expect("git config should run");
        assert!(
            user_name_output.status.success(),
            "git config should succeed"
        );
        let user_email_output = Command::new("git")
            .current_dir(&repo_root)
            .args(["config", "user.email", "grove@example.com"])
            .output()
            .expect("git config should run");
        assert!(
            user_email_output.status.success(),
            "git config should succeed"
        );
        fs::write(repo_root.join("README.md"), "hello\n").expect("readme should write");
        let add_output = Command::new("git")
            .current_dir(&repo_root)
            .args(["add", "README.md"])
            .output()
            .expect("git add should run");
        assert!(add_output.status.success(), "git add should succeed");
        let commit_output = Command::new("git")
            .current_dir(&repo_root)
            .args(["commit", "-m", "init"])
            .output()
            .expect("git commit should run");
        assert!(commit_output.status.success(), "git commit should succeed");

        let bootstrap = bootstrap_task_data_for_root(
            tasks_root.as_path(),
            &[ProjectConfig {
                name: "mcp".to_string(),
                path: repo_root.clone(),
                defaults: Default::default(),
            }],
        );

        assert_eq!(bootstrap.tasks.len(), 1);
        assert_eq!(bootstrap.tasks[0].worktrees[0].repository_path, repo_root);
        assert!(
            tasks_root
                .join("mcp")
                .join(".grove")
                .join("task.toml")
                .exists()
        );
    }
}
