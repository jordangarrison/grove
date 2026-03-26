use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::domain::PermissionMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeName {
    Monokai,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    #[default]
    CatppuccinMocha,
    RosePine,
    RosePineMoon,
    RosePineDawn,
}

impl ThemeName {
    pub const fn config_key(self) -> &'static str {
        match self {
            Self::Monokai => "monokai",
            Self::CatppuccinLatte => "catppuccin-latte",
            Self::CatppuccinFrappe => "catppuccin-frappe",
            Self::CatppuccinMacchiato => "catppuccin-macchiato",
            Self::CatppuccinMocha => "catppuccin-mocha",
            Self::RosePine => "rose-pine",
            Self::RosePineMoon => "rose-pine-moon",
            Self::RosePineDawn => "rose-pine-dawn",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroveConfig {
    #[serde(default = "default_sidebar_width_pct")]
    pub sidebar_width_pct: u16,
    #[serde(default)]
    pub theme: ThemeName,
    #[serde(default)]
    pub projects: Vec<ProjectConfig>,
    #[serde(default)]
    pub task_order: Vec<String>,
    #[serde(default)]
    pub attention_acks: Vec<WorkspaceAttentionAckConfig>,
    #[serde(default)]
    pub hidden_base_project_paths: Vec<PathBuf>,
    #[serde(default)]
    pub launch_permission_mode: PermissionMode,
}

const fn default_sidebar_width_pct() -> u16 {
    33
}

impl Default for GroveConfig {
    fn default() -> Self {
        Self {
            sidebar_width_pct: default_sidebar_width_pct(),
            theme: ThemeName::default(),
            projects: Vec::new(),
            task_order: Vec::new(),
            attention_acks: Vec::new(),
            hidden_base_project_paths: Vec::new(),
            launch_permission_mode: PermissionMode::Default,
        }
    }
}

impl GroveConfig {
    pub fn global_settings(&self) -> GlobalSettings {
        GlobalSettings {
            sidebar_width_pct: self.sidebar_width_pct,
            theme: self.theme,
            launch_permission_mode: self.launch_permission_mode,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAttentionAckConfig {
    pub workspace_path: PathBuf,
    pub marker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub defaults: ProjectDefaults,
}

pub type RepositoryConfig = ProjectConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProjectDefaults {
    #[serde(default)]
    pub base_branch: String,
    #[serde(default)]
    pub workspace_init_command: String,
    #[serde(default)]
    pub agent_env: AgentEnvDefaults,
}

pub type RepositoryDefaults = ProjectDefaults;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AgentEnvDefaults {
    #[serde(default)]
    pub claude: Vec<String>,
    #[serde(default)]
    pub codex: Vec<String>,
    #[serde(default)]
    pub opencode: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedConfig {
    pub path: PathBuf,
    pub config: GroveConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalSettings {
    #[serde(default = "default_sidebar_width_pct")]
    pub sidebar_width_pct: u16,
    #[serde(default)]
    pub theme: ThemeName,
    #[serde(default)]
    pub launch_permission_mode: PermissionMode,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            sidebar_width_pct: default_sidebar_width_pct(),
            theme: ThemeName::default(),
            launch_permission_mode: PermissionMode::Default,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProjectsState {
    #[serde(default)]
    pub projects: Vec<ProjectConfig>,
    #[serde(default)]
    pub task_order: Vec<String>,
    #[serde(default)]
    pub attention_acks: Vec<WorkspaceAttentionAckConfig>,
    #[serde(default)]
    pub hidden_base_project_paths: Vec<PathBuf>,
}

fn config_directory() -> Option<PathBuf> {
    if let Some(path) = dirs::config_dir() {
        return Some(path.join("grove"));
    }

    dirs::home_dir().map(|path| path.join(".config").join("grove"))
}

pub fn config_path() -> Option<PathBuf> {
    config_directory().map(|path| path.join("config.toml"))
}

pub fn projects_path() -> Option<PathBuf> {
    config_path().map(|path| projects_path_for(path.as_path()))
}

pub fn projects_path_for(config_path: &Path) -> PathBuf {
    let file_name = config_path.file_name().map_or_else(
        || OsString::from("projects.toml"),
        |name| {
            if name == "config.toml" {
                OsString::from("projects.toml")
            } else {
                let mut value = name.to_os_string();
                value.push(".projects.toml");
                value
            }
        },
    );
    config_path.with_file_name(file_name)
}

pub fn load() -> Result<LoadedConfig, String> {
    let path = config_path().ok_or_else(|| "cannot resolve config path".to_string())?;
    let config = load_from_path(&path)?;
    Ok(LoadedConfig { path, config })
}

pub fn load_from_path(path: &Path) -> Result<GroveConfig, String> {
    let settings = load_global_from_path(path)?;
    let projects = load_projects_from_path(&projects_path_for(path))?;
    Ok(GroveConfig {
        sidebar_width_pct: settings.sidebar_width_pct,
        theme: settings.theme,
        projects: projects.projects,
        task_order: projects.task_order,
        attention_acks: projects.attention_acks,
        hidden_base_project_paths: projects.hidden_base_project_paths,
        launch_permission_mode: settings.launch_permission_mode,
    })
}

pub fn load_global_from_path(path: &Path) -> Result<GlobalSettings, String> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(GlobalSettings::default());
        }
        Err(error) => return Err(format!("global config read failed: {error}")),
    };

    toml::from_str::<GlobalSettings>(&raw)
        .map_err(|error| format!("global config parse failed: {error}"))
}

pub fn load_projects_from_path(path: &Path) -> Result<ProjectsState, String> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ProjectsState::default());
        }
        Err(error) => return Err(format!("projects config read failed: {error}")),
    };

    toml::from_str::<ProjectsState>(&raw)
        .map_err(|error| format!("projects config parse failed: {error}"))
}

pub fn save_global_to_path(path: &Path, settings: &GlobalSettings) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err("global config path missing parent directory".to_string());
    };

    fs::create_dir_all(parent)
        .map_err(|error| format!("global config directory create failed: {error}"))?;
    let encoded = toml::to_string_pretty(settings)
        .map_err(|error| format!("global config encode failed: {error}"))?;
    fs::write(path, encoded).map_err(|error| format!("global config write failed: {error}"))
}

pub fn save_projects_to_path(
    path: &Path,
    projects: &[ProjectConfig],
    task_order: &[String],
    attention_acks: &[WorkspaceAttentionAckConfig],
    hidden_base_project_paths: &[PathBuf],
) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err("projects config path missing parent directory".to_string());
    };

    fs::create_dir_all(parent)
        .map_err(|error| format!("projects config directory create failed: {error}"))?;
    let projects_state = ProjectsState {
        projects: projects.to_vec(),
        task_order: task_order.to_vec(),
        attention_acks: attention_acks.to_vec(),
        hidden_base_project_paths: hidden_base_project_paths.to_vec(),
    };
    let encoded = toml::to_string_pretty(&projects_state)
        .map_err(|error| format!("projects config encode failed: {error}"))?;
    fs::write(path, encoded).map_err(|error| format!("projects config write failed: {error}"))
}

pub fn save_projects_state_from_config_path(
    path: &Path,
    config: &GroveConfig,
) -> Result<(), String> {
    let projects_path = projects_path_for(path);
    save_projects_to_path(
        &projects_path,
        &config.projects,
        &config.task_order,
        &config.attention_acks,
        &config.hidden_base_project_paths,
    )
}

pub fn save_to_path(path: &Path, config: &GroveConfig) -> Result<(), String> {
    save_global_to_path(path, &config.global_settings())?;
    save_projects_state_from_config_path(path, config)
}

#[cfg(test)]
mod tests {
    use super::{
        AgentEnvDefaults, GlobalSettings, GroveConfig, PermissionMode, ProjectConfig,
        ProjectDefaults, RepositoryConfig, RepositoryDefaults, ThemeName, load_from_path,
        projects_path_for, save_global_to_path, save_projects_to_path, save_to_path,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(label: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos();
        let pid = std::process::id();
        std::env::temp_dir().join(format!("grove-config-{label}-{pid}-{timestamp}.toml"))
    }

    fn cleanup_files(config_path: &Path) {
        let _ = fs::remove_file(config_path);
        let _ = fs::remove_file(projects_path_for(config_path));
    }

    #[test]
    fn missing_config_uses_defaults() {
        let path = unique_temp_path("missing");
        let config = load_from_path(&path).expect("missing path should default");
        assert_eq!(
            config,
            GroveConfig {
                sidebar_width_pct: 33,
                theme: ThemeName::CatppuccinMocha,
                projects: Vec::new(),
                task_order: Vec::new(),
                attention_acks: Vec::new(),
                hidden_base_project_paths: Vec::new(),
                launch_permission_mode: PermissionMode::Default,
            }
        );
    }

    #[test]
    fn repository_aliases_share_project_config_shape() {
        let repository = RepositoryConfig {
            name: "grove".to_string(),
            path: PathBuf::from("/repos/grove"),
            defaults: RepositoryDefaults {
                base_branch: "main".to_string(),
                workspace_init_command: "direnv allow".to_string(),
                agent_env: AgentEnvDefaults::default(),
            },
        };

        assert_eq!(repository.name, "grove");
        assert_eq!(repository.path, PathBuf::from("/repos/grove"));
        assert_eq!(repository.defaults.base_branch, "main");
    }

    #[test]
    fn save_and_load_round_trip() {
        let path = unique_temp_path("roundtrip");
        let config = GroveConfig {
            sidebar_width_pct: 52,
            theme: ThemeName::Monokai,
            projects: vec![ProjectConfig {
                name: "grove".to_string(),
                path: PathBuf::from("/repos/grove"),
                defaults: ProjectDefaults {
                    base_branch: "develop".to_string(),
                    workspace_init_command: "direnv exec . true".to_string(),
                    agent_env: AgentEnvDefaults {
                        claude: vec!["CLAUDE_CONFIG_DIR=~/.claude-work".to_string()],
                        codex: vec!["CODEX_CONFIG_DIR=~/.codex-work".to_string()],
                        opencode: Vec::new(),
                    },
                },
            }],
            task_order: vec!["grove".to_string(), "task-workflow".to_string()],
            attention_acks: Vec::new(),
            hidden_base_project_paths: vec![PathBuf::from("/repos/hidden")],
            launch_permission_mode: PermissionMode::Unsafe,
        };
        save_to_path(&path, &config).expect("config should save");

        let loaded = load_from_path(&path).expect("config should load");
        assert_eq!(loaded, config);

        cleanup_files(path.as_path());
    }

    #[test]
    fn load_old_config_without_projects_defaults_to_empty_projects() {
        let path = unique_temp_path("legacy");
        fs::write(&path, "multiplexer = \"tmux\"\n").expect("fixture should write");

        let loaded = load_from_path(&path).expect("legacy config should load");
        assert_eq!(loaded.sidebar_width_pct, 33);
        assert_eq!(loaded.theme, ThemeName::CatppuccinMocha);
        assert_eq!(loaded.projects, Vec::<ProjectConfig>::new());
        assert_eq!(loaded.task_order, Vec::<String>::new());
        assert_eq!(loaded.hidden_base_project_paths, Vec::<PathBuf>::new());

        cleanup_files(path.as_path());
    }

    #[test]
    fn load_project_without_defaults_uses_project_defaults_fallback() {
        let path = unique_temp_path("project-defaults");
        let projects_path = projects_path_for(path.as_path());
        fs::write(
            &projects_path,
            "[[projects]]\nname = \"grove\"\npath = \"/repos/grove\"\n",
        )
        .expect("fixture should write");

        let loaded = load_from_path(&path).expect("legacy project config should load");
        assert_eq!(loaded.projects.len(), 1);
        assert_eq!(loaded.sidebar_width_pct, 33);
        assert_eq!(loaded.theme, ThemeName::CatppuccinMocha);
        assert_eq!(loaded.attention_acks, Vec::new());
        assert_eq!(loaded.task_order, Vec::<String>::new());
        assert_eq!(loaded.hidden_base_project_paths, Vec::<PathBuf>::new());
        assert_eq!(loaded.launch_permission_mode, PermissionMode::Default);
        assert_eq!(loaded.projects[0].defaults.base_branch, "");
        assert_eq!(loaded.projects[0].defaults.workspace_init_command, "");
        assert_eq!(
            loaded.projects[0].defaults.agent_env,
            AgentEnvDefaults::default()
        );

        cleanup_files(path.as_path());
    }

    #[test]
    fn save_global_to_path_does_not_clear_projects_state() {
        let path = unique_temp_path("global-only-save");
        let projects_path = projects_path_for(path.as_path());
        let initial = GroveConfig {
            sidebar_width_pct: 33,
            theme: ThemeName::CatppuccinMocha,
            projects: vec![ProjectConfig {
                name: "grove".to_string(),
                path: PathBuf::from("/repos/grove"),
                defaults: ProjectDefaults::default(),
            }],
            task_order: vec!["grove".to_string()],
            attention_acks: Vec::new(),
            hidden_base_project_paths: vec![PathBuf::from("/repos/hidden")],
            launch_permission_mode: PermissionMode::Default,
        };
        save_projects_to_path(
            &projects_path,
            &initial.projects,
            &initial.task_order,
            &initial.attention_acks,
            &initial.hidden_base_project_paths,
        )
        .expect("projects should save");
        let updated = GlobalSettings {
            sidebar_width_pct: 48,
            theme: ThemeName::CatppuccinLatte,
            launch_permission_mode: PermissionMode::Unsafe,
        };
        save_global_to_path(&path, &updated).expect("global settings should save");

        let loaded = load_from_path(&path).expect("combined config should load");
        assert_eq!(loaded.sidebar_width_pct, 48);
        assert_eq!(loaded.theme, ThemeName::CatppuccinLatte);
        assert_eq!(loaded.launch_permission_mode, PermissionMode::Unsafe);
        assert_eq!(loaded.projects.len(), 1);
        assert_eq!(loaded.projects[0].name, "grove");
        assert_eq!(loaded.task_order, vec!["grove".to_string()]);

        cleanup_files(path.as_path());
    }

    #[test]
    fn save_projects_to_path_does_not_clear_global_settings() {
        let path = unique_temp_path("projects-only-save");
        let projects_path = projects_path_for(path.as_path());
        let settings = GlobalSettings {
            sidebar_width_pct: 61,
            theme: ThemeName::CatppuccinFrappe,
            launch_permission_mode: PermissionMode::Unsafe,
        };
        save_global_to_path(&path, &settings).expect("global settings should save");
        let projects = vec![ProjectConfig {
            name: "grove".to_string(),
            path: PathBuf::from("/repos/grove"),
            defaults: ProjectDefaults::default(),
        }];
        let task_order = vec!["task-workflow".to_string(), "grove".to_string()];
        save_projects_to_path(&projects_path, &projects, &task_order, &[], &[])
            .expect("projects state should save");

        let loaded = load_from_path(&path).expect("combined config should load");
        assert_eq!(loaded.sidebar_width_pct, 61);
        assert_eq!(loaded.theme, ThemeName::CatppuccinFrappe);
        assert_eq!(loaded.launch_permission_mode, PermissionMode::Unsafe);
        assert_eq!(loaded.projects.len(), 1);
        assert_eq!(loaded.projects[0].name, "grove");
        assert_eq!(loaded.task_order, task_order);

        cleanup_files(path.as_path());
    }

    #[test]
    fn task_order_round_trips_through_projects_state() {
        let path = unique_temp_path("task-order-roundtrip");
        let projects_path = projects_path_for(path.as_path());
        let projects = vec![ProjectConfig {
            name: "grove".to_string(),
            path: PathBuf::from("/repos/grove"),
            defaults: ProjectDefaults::default(),
        }];
        let task_order = vec!["task-workflow".to_string(), "grove".to_string()];

        save_projects_to_path(&projects_path, &projects, &task_order, &[], &[])
            .expect("projects state should save");

        let loaded = load_from_path(&path).expect("combined config should load");
        assert_eq!(loaded.task_order, task_order);

        cleanup_files(path.as_path());
    }
}
