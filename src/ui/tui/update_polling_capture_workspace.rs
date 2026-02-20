use super::*;

impl GroveApp {
    pub(super) fn apply_workspace_status_capture(&mut self, capture: WorkspaceStatusCapture) {
        let supported_agent = capture.supported_agent;
        let Some(workspace_index) = self
            .state
            .workspaces
            .iter()
            .position(|workspace| workspace.path == capture.workspace_path)
        else {
            return;
        };

        match capture.result {
            Ok(output) => {
                let (_, cleaned_output) =
                    self.capture_changed_cleaned_for_workspace(&capture.workspace_path, &output);
                let workspace_path = self.state.workspaces[workspace_index].path.clone();
                let previous_status = self.state.workspaces[workspace_index].status;
                let previous_orphaned = self.state.workspaces[workspace_index].is_orphaned;
                let workspace_agent = self.state.workspaces[workspace_index].agent;
                let workspace_is_main = self.state.workspaces[workspace_index].is_main;
                let next_status = detect_status_with_session_override(
                    cleaned_output.as_str(),
                    SessionActivity::Active,
                    workspace_is_main,
                    true,
                    supported_agent,
                    workspace_agent,
                    &workspace_path,
                );
                let workspace = &mut self.state.workspaces[workspace_index];
                workspace.status = next_status;
                workspace.is_orphaned = false;
                self.track_workspace_status_transition(
                    &workspace_path,
                    previous_status,
                    next_status,
                    previous_orphaned,
                    false,
                );
            }
            Err(error) => {
                if tmux_capture_error_indicates_missing_session(&error) {
                    let previous_status = self.state.workspaces[workspace_index].status;
                    let previous_orphaned = self.state.workspaces[workspace_index].is_orphaned;
                    let workspace_is_main = self.state.workspaces[workspace_index].is_main;
                    let previously_had_live_session = previous_status.has_session();
                    let next_status = if workspace_is_main {
                        WorkspaceStatus::Main
                    } else {
                        WorkspaceStatus::Idle
                    };
                    let next_orphaned = if workspace_is_main {
                        false
                    } else {
                        previously_had_live_session || previous_orphaned
                    };
                    let workspace = &mut self.state.workspaces[workspace_index];
                    workspace.status = next_status;
                    workspace.is_orphaned = next_orphaned;
                    self.clear_status_tracking_for_workspace_path(&capture.workspace_path);
                    self.track_workspace_status_transition(
                        &capture.workspace_path,
                        previous_status,
                        next_status,
                        previous_orphaned,
                        next_orphaned,
                    );
                }
            }
        }
    }
}
