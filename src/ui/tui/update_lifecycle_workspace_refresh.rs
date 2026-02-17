use super::*;

impl GroveApp {
    fn auto_launch_pending_workspace_shell(&mut self) {
        let Some(pending_path) = self.pending_auto_shell_workspace_path.clone() else {
            return;
        };
        self.pending_auto_shell_workspace_path = None;

        let selected_matches = self
            .state
            .selected_workspace()
            .is_some_and(|workspace| workspace.path == pending_path);
        if !selected_matches {
            return;
        }

        let _ = self.ensure_workspace_shell_session_for_selected_workspace(true);
    }

    pub(super) fn refresh_workspaces(&mut self, preferred_workspace_path: Option<PathBuf>) {
        if !self.tmux_input.supports_background_launch() {
            self.refresh_workspaces_sync(preferred_workspace_path);
            return;
        }

        if self.refresh_in_flight {
            return;
        }

        let target_path = preferred_workspace_path.or_else(|| self.selected_workspace_path());
        let multiplexer = self.multiplexer;
        let projects = self.projects.clone();
        self.refresh_in_flight = true;
        self.queue_cmd(Cmd::task(move || {
            let bootstrap = bootstrap_data_for_projects(&projects, multiplexer);
            Msg::RefreshWorkspacesCompleted(RefreshWorkspacesCompletion {
                preferred_workspace_path: target_path,
                bootstrap,
            })
        }));
    }

    fn refresh_workspaces_sync(&mut self, preferred_workspace_path: Option<PathBuf>) {
        let target_path = preferred_workspace_path.or_else(|| self.selected_workspace_path());
        let previous_mode = self.state.mode;
        let previous_focus = self.state.focus;
        let bootstrap = bootstrap_data_for_projects(&self.projects, self.multiplexer);

        self.repo_name = bootstrap.repo_name;
        self.discovery_state = bootstrap.discovery_state;
        self.state = AppState::new(bootstrap.workspaces);
        if let Some(path) = target_path
            && let Some(index) = self
                .state
                .workspaces
                .iter()
                .position(|workspace| workspace.path == path)
        {
            self.state.selected_index = index;
        }
        self.state.mode = previous_mode;
        self.state.focus = previous_focus;
        self.clear_agent_activity_tracking();
        self.clear_status_tracking();
        self.auto_launch_pending_workspace_shell();
        self.poll_preview();
    }

    pub(super) fn apply_refresh_workspaces_completion(
        &mut self,
        completion: RefreshWorkspacesCompletion,
    ) {
        let previous_mode = self.state.mode;
        let previous_focus = self.state.focus;

        self.repo_name = completion.bootstrap.repo_name;
        self.discovery_state = completion.bootstrap.discovery_state;
        self.state = AppState::new(completion.bootstrap.workspaces);
        if let Some(path) = completion.preferred_workspace_path
            && let Some(index) = self
                .state
                .workspaces
                .iter()
                .position(|workspace| workspace.path == path)
        {
            self.state.selected_index = index;
        }
        self.state.mode = previous_mode;
        self.state.focus = previous_focus;
        self.refresh_in_flight = false;
        self.clear_agent_activity_tracking();
        self.clear_status_tracking();
        self.auto_launch_pending_workspace_shell();
        self.poll_preview();
    }
}
