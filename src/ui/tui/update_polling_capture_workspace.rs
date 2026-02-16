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
                self.capture_changed_cleaned_for_workspace(&capture.workspace_path, &output);
                let workspace_path = self.state.workspaces[workspace_index].path.clone();
                let workspace_agent = self.state.workspaces[workspace_index].agent;
                let workspace_is_main = self.state.workspaces[workspace_index].is_main;
                let workspace = &mut self.state.workspaces[workspace_index];
                workspace.status = detect_status_with_session_override(
                    output.as_str(),
                    SessionActivity::Active,
                    workspace_is_main,
                    true,
                    supported_agent,
                    workspace_agent,
                    &workspace_path,
                );
                workspace.is_orphaned = false;
            }
            Err(error) => {
                if tmux_capture_error_indicates_missing_session(&error) {
                    let workspace = &mut self.state.workspaces[workspace_index];
                    let previously_had_live_session = workspace.status.has_session();
                    workspace.status = if workspace.is_main {
                        WorkspaceStatus::Main
                    } else {
                        WorkspaceStatus::Idle
                    };
                    workspace.is_orphaned = if workspace.is_main {
                        false
                    } else {
                        previously_had_live_session || workspace.is_orphaned
                    };
                    self.clear_status_tracking_for_workspace_path(&capture.workspace_path);
                }
            }
        }
    }
}
