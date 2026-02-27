use super::{
    AgentEnvDefaults, GroveConfig, ProjectConfig, ProjectDefaults, load_from_path, save_to_path,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_path(label: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    let pid = std::process::id();
    std::env::temp_dir().join(format!("grove-config-{label}-{pid}-{timestamp}.toml"))
}

#[test]
fn missing_config_uses_defaults() {
    let path = unique_temp_path("missing");
    let config = load_from_path(&path).expect("missing path should default");
    assert_eq!(
        config,
        GroveConfig {
            sidebar_width_pct: 33,
            projects: Vec::new(),
            attention_acks: Vec::new(),
            launch_skip_permissions: false,
        }
    );
}

#[test]
fn save_and_load_round_trip() {
    let path = unique_temp_path("roundtrip");
    let config = GroveConfig {
        sidebar_width_pct: 52,
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
                ..ProjectDefaults::default()
            },
        }],
        attention_acks: Vec::new(),
        launch_skip_permissions: true,
    };
    save_to_path(&path, &config).expect("config should save");

    let loaded = load_from_path(&path).expect("config should load");
    assert_eq!(loaded, config);

    let _ = fs::remove_file(path);
}

#[test]
fn load_old_config_without_projects_defaults_to_empty_projects() {
    let path = unique_temp_path("legacy");
    fs::write(&path, "multiplexer = \"tmux\"\n").expect("fixture should write");

    let loaded = load_from_path(&path).expect("legacy config should load");
    assert_eq!(loaded.sidebar_width_pct, 33);
    assert_eq!(loaded.projects, Vec::<ProjectConfig>::new());

    let _ = fs::remove_file(path);
}

#[test]
fn load_project_without_defaults_uses_project_defaults_fallback() {
    let path = unique_temp_path("project-defaults");
    fs::write(
        &path,
        "multiplexer = \"tmux\"\n[[projects]]\nname = \"grove\"\npath = \"/repos/grove\"\n",
    )
    .expect("fixture should write");

    let loaded = load_from_path(&path).expect("legacy project config should load");
    assert_eq!(loaded.projects.len(), 1);
    assert_eq!(loaded.sidebar_width_pct, 33);
    assert_eq!(loaded.attention_acks, Vec::new());
    assert!(!loaded.launch_skip_permissions);
    assert_eq!(loaded.projects[0].defaults.base_branch, "");
    assert_eq!(loaded.projects[0].defaults.workspace_init_command, "");
    assert_eq!(
        loaded.projects[0].defaults.agent_env,
        AgentEnvDefaults::default()
    );

    let _ = fs::remove_file(path);
}

#[test]
fn load_project_with_legacy_setup_commands_migrates_first_to_workspace_init_command() {
    let path = unique_temp_path("legacy-setup-migration");
    fs::write(
        &path,
        "[[projects]]\nname = \"grove\"\npath = \"/repos/grove\"\n[projects.defaults]\nsetup_commands = [\"\", \"direnv allow\", \"nix develop -c true\"]\n",
    )
    .expect("fixture should write");

    let loaded = load_from_path(&path).expect("legacy setup should load");
    assert_eq!(loaded.projects.len(), 1);
    assert_eq!(
        loaded.projects[0].defaults.workspace_init_command,
        "direnv allow"
    );

    let _ = fs::remove_file(path);
}
