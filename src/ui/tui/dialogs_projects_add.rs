use super::*;

impl GroveApp {
    fn normalized_project_path(raw: &str) -> PathBuf {
        if let Some(stripped) = raw.strip_prefix("~/")
            && let Some(home) = dirs::home_dir()
        {
            return home.join(stripped);
        }
        PathBuf::from(raw)
    }

    fn save_projects_config(&self) -> Result<(), String> {
        let config = GroveConfig {
            multiplexer: self.multiplexer,
            projects: self.projects.clone(),
        };
        crate::infrastructure::config::save_to_path(&self.config_path, &config)
    }

    pub(super) fn add_project_from_dialog(&mut self) {
        let Some(project_dialog) = self.project_dialog.as_ref() else {
            return;
        };
        let Some(add_dialog) = project_dialog.add_dialog.as_ref() else {
            return;
        };

        let path_input = add_dialog.path.trim();
        if path_input.is_empty() {
            self.show_toast("project path is required", true);
            return;
        }
        let normalized = Self::normalized_project_path(path_input);
        let canonical = match normalized.canonicalize() {
            Ok(path) => path,
            Err(error) => {
                self.show_toast(format!("invalid project path: {error}"), true);
                return;
            }
        };

        let repo_root_output = Command::new("git")
            .current_dir(&canonical)
            .args(["rev-parse", "--show-toplevel"])
            .output();
        let repo_root = match repo_root_output {
            Ok(output) if output.status.success() => {
                let raw = String::from_utf8(output.stdout).unwrap_or_default();
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    canonical.clone()
                } else {
                    PathBuf::from(trimmed)
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                self.show_toast(format!("not a git repository: {stderr}"), true);
                return;
            }
            Err(error) => {
                self.show_toast(format!("git check failed: {error}"), true);
                return;
            }
        };
        let repo_root = repo_root.canonicalize().unwrap_or(repo_root);

        if self
            .projects
            .iter()
            .any(|project| project_paths_equal(&project.path, &repo_root))
        {
            self.show_toast("project already exists", true);
            return;
        }

        let project_name = if add_dialog.name.trim().is_empty() {
            project_display_name(&repo_root)
        } else {
            add_dialog.name.trim().to_string()
        };
        self.projects.push(ProjectConfig {
            name: project_name.clone(),
            path: repo_root.clone(),
        });
        if let Err(error) = self.save_projects_config() {
            self.show_toast(format!("project save failed: {error}"), true);
            return;
        }

        if let Some(dialog) = self.project_dialog.as_mut() {
            dialog.add_dialog = None;
        }
        self.refresh_project_dialog_filtered();
        self.refresh_workspaces(None);
        self.show_toast(format!("project '{}' added", project_name), false);
    }
}
