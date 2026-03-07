use std::fs;
use std::path::{Path, PathBuf};

use crate::domain::{AgentType, Task, Worktree};
use crate::infrastructure::config::RepositoryConfig;
use crate::infrastructure::task_manifest::encode_task_manifest;

use crate::application::workspace_lifecycle::{
    GitCommandRunner, SetupCommandRunner, SetupScriptRunner,
};

#[path = "task_lifecycle/create.rs"]
mod create;

const GROVE_SETUP_SCRIPT_FILE: &str = ".grove/setup.sh";
const TASK_MANIFEST_FILE: &str = ".grove/task.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskLifecycleError {
    EmptyTaskName,
    InvalidTaskName,
    EmptyBaseBranch,
    EmptyRepositories,
    HomeDirectoryUnavailable,
    RepositoryNameUnavailable,
    TaskInvalid(String),
    TaskManifest(String),
    GitCommandFailed(String),
    Io(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTaskRequest {
    pub task_name: String,
    pub repositories: Vec<RepositoryConfig>,
    pub base_branch: String,
    pub agent: AgentType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTaskResult {
    pub task_root: PathBuf,
    pub task: Task,
    pub warnings: Vec<String>,
}

impl CreateTaskRequest {
    pub fn validate(&self) -> Result<(), TaskLifecycleError> {
        if self.task_name.trim().is_empty() {
            return Err(TaskLifecycleError::EmptyTaskName);
        }
        if !task_name_is_valid(self.task_name.as_str()) {
            return Err(TaskLifecycleError::InvalidTaskName);
        }
        if self.base_branch.trim().is_empty() {
            return Err(TaskLifecycleError::EmptyBaseBranch);
        }
        if self.repositories.is_empty() {
            return Err(TaskLifecycleError::EmptyRepositories);
        }
        Ok(())
    }
}

pub fn create_task(
    request: &CreateTaskRequest,
    git_runner: &impl GitCommandRunner,
    setup_script_runner: &impl SetupScriptRunner,
    setup_command_runner: &impl SetupCommandRunner,
) -> Result<CreateTaskResult, TaskLifecycleError> {
    let home_directory = dirs::home_dir().ok_or(TaskLifecycleError::HomeDirectoryUnavailable)?;
    let tasks_root = home_directory.join(".grove").join("tasks");
    create_task_in_root(
        tasks_root.as_path(),
        request,
        git_runner,
        setup_script_runner,
        setup_command_runner,
    )
}

pub fn create_task_in_root(
    tasks_root: &Path,
    request: &CreateTaskRequest,
    git_runner: &impl GitCommandRunner,
    setup_script_runner: &impl SetupScriptRunner,
    setup_command_runner: &impl SetupCommandRunner,
) -> Result<CreateTaskResult, TaskLifecycleError> {
    create::create_task_in_root(
        tasks_root,
        request,
        git_runner,
        setup_script_runner,
        setup_command_runner,
    )
}

fn task_name_is_valid(name: &str) -> bool {
    name.chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_')
}

fn task_manifest_path(task_root: &Path) -> PathBuf {
    task_root.join(TASK_MANIFEST_FILE)
}

fn write_task_manifest(task_root: &Path, task: &Task) -> Result<(), TaskLifecycleError> {
    let manifest_path = task_manifest_path(task_root);
    let Some(manifest_parent) = manifest_path.parent() else {
        return Err(TaskLifecycleError::TaskManifest(
            "task manifest path missing parent".to_string(),
        ));
    };
    fs::create_dir_all(manifest_parent)
        .map_err(|error| TaskLifecycleError::Io(error.to_string()))?;
    let encoded = encode_task_manifest(task).map_err(TaskLifecycleError::TaskManifest)?;
    fs::write(manifest_path, encoded).map_err(|error| TaskLifecycleError::Io(error.to_string()))
}

fn repo_directory_name(repository: &RepositoryConfig) -> Result<String, TaskLifecycleError> {
    repository
        .path
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .ok_or(TaskLifecycleError::RepositoryNameUnavailable)
}

fn create_task_domain(
    task_name: &str,
    task_root: &Path,
    worktrees: Vec<Worktree>,
) -> Result<Task, TaskLifecycleError> {
    Task::try_new(
        task_name.to_string(),
        task_name.to_string(),
        task_root.to_path_buf(),
        task_name.to_string(),
        worktrees,
    )
    .map_err(|error| TaskLifecycleError::TaskInvalid(format!("{error:?}")))
}

#[cfg(test)]
mod tests {
    use super::{CreateTaskRequest, create_task_in_root, repo_directory_name, task_manifest_path};
    use crate::application::workspace_lifecycle::{
        GitCommandRunner, SetupCommandContext, SetupCommandRunner, SetupScriptContext,
        SetupScriptRunner,
    };
    use crate::domain::AgentType;
    use crate::infrastructure::config::{ProjectDefaults, RepositoryConfig};
    use std::cell::RefCell;
    use std::fs;
    use std::path::{Path, PathBuf};
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
                "grove-task-lifecycle-{label}-{}-{timestamp}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("test dir should be created");
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[derive(Default)]
    struct StubGitRunner {
        calls: RefCell<Vec<(PathBuf, Vec<String>)>>,
    }

    impl StubGitRunner {
        fn calls(&self) -> Vec<(PathBuf, Vec<String>)> {
            self.calls.borrow().clone()
        }
    }

    impl GitCommandRunner for StubGitRunner {
        fn run(&self, repo_root: &Path, args: &[String]) -> Result<(), String> {
            self.calls
                .borrow_mut()
                .push((repo_root.to_path_buf(), args.to_vec()));
            Ok(())
        }
    }

    #[derive(Default)]
    struct StubSetupRunner;

    impl SetupScriptRunner for StubSetupRunner {
        fn run(&self, _context: &SetupScriptContext) -> Result<(), String> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct StubSetupCommandRunner;

    impl SetupCommandRunner for StubSetupCommandRunner {
        fn run(&self, _context: &SetupCommandContext, _command: &str) -> Result<(), String> {
            Ok(())
        }
    }

    fn repository(path: PathBuf) -> RepositoryConfig {
        RepositoryConfig {
            name: path
                .file_name()
                .and_then(|value| value.to_str())
                .map(str::to_string)
                .unwrap_or_else(|| "repo".to_string()),
            path,
            defaults: ProjectDefaults::default(),
        }
    }

    #[test]
    fn repo_directory_name_uses_repository_path_basename() {
        let repository = repository(PathBuf::from("/repos/terraform-fastly"));
        assert_eq!(
            repo_directory_name(&repository).expect("repo directory should resolve"),
            "terraform-fastly"
        );
    }

    #[test]
    fn create_task_builds_one_worktree_per_repository_under_task_root() {
        let temp = TestDir::new("create");
        let tasks_root = temp.path.join("tasks");
        let flohome = temp.path.join("repos").join("flohome");
        let fastly = temp.path.join("repos").join("terraform-fastly");
        fs::create_dir_all(&flohome).expect("flohome repo should exist");
        fs::create_dir_all(&fastly).expect("fastly repo should exist");
        fs::write(flohome.join(".env"), "APP=1\n").expect("env should write");
        fs::write(fastly.join(".env.local"), "FASTLY=1\n").expect("env local should write");

        let request = CreateTaskRequest {
            task_name: "flohome-launch".to_string(),
            repositories: vec![repository(flohome.clone()), repository(fastly.clone())],
            base_branch: "main".to_string(),
            agent: AgentType::Codex,
        };
        let git = StubGitRunner::default();
        let setup = StubSetupRunner;
        let setup_command = StubSetupCommandRunner;

        let result =
            create_task_in_root(tasks_root.as_path(), &request, &git, &setup, &setup_command)
                .expect("task should create");

        assert_eq!(result.task_root, tasks_root.join("flohome-launch"));
        assert_eq!(result.task.worktrees.len(), 2);
        assert!(result.task.worktrees[0].path.starts_with(&result.task_root));
        assert!(result.task.worktrees[1].path.starts_with(&result.task_root));
        assert!(task_manifest_path(&result.task_root).exists());
        assert_eq!(
            fs::read_to_string(result.task.worktrees[0].path.join(".grove/base"))
                .expect("base marker should exist")
                .trim(),
            "main"
        );
        assert_eq!(
            fs::read_to_string(result.task.worktrees[0].path.join(".env"))
                .expect("env should copy"),
            "APP=1\n"
        );
        assert_eq!(
            fs::read_to_string(result.task.worktrees[1].path.join(".env.local"))
                .expect("env local should copy"),
            "FASTLY=1\n"
        );
        assert_eq!(
            git.calls(),
            vec![
                (
                    flohome.clone(),
                    vec![
                        "worktree".to_string(),
                        "add".to_string(),
                        "-b".to_string(),
                        "flohome-launch".to_string(),
                        tasks_root
                            .join("flohome-launch")
                            .join("flohome")
                            .to_string_lossy()
                            .to_string(),
                        "main".to_string(),
                    ],
                ),
                (
                    fastly.clone(),
                    vec![
                        "worktree".to_string(),
                        "add".to_string(),
                        "-b".to_string(),
                        "flohome-launch".to_string(),
                        tasks_root
                            .join("flohome-launch")
                            .join("terraform-fastly")
                            .to_string_lossy()
                            .to_string(),
                        "main".to_string(),
                    ],
                ),
            ]
        );
    }
}
