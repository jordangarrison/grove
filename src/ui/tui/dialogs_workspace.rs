use super::*;

impl GroveApp {
    pub(super) fn handle_delete_dialog_key(&mut self, key_event: KeyEvent) {
        if self.delete_in_flight {
            return;
        }
        let no_modifiers = key_event.modifiers.is_empty();
        match key_event.code {
            KeyCode::Escape => {
                self.log_dialog_event("delete", "dialog_cancelled");
                self.delete_dialog = None;
                return;
            }
            KeyCode::Char('q') if no_modifiers => {
                self.log_dialog_event("delete", "dialog_cancelled");
                self.delete_dialog = None;
                return;
            }
            KeyCode::Char('D') if no_modifiers => {
                self.confirm_delete_dialog();
                return;
            }
            _ => {}
        }

        let mut confirm_delete = false;
        let mut cancel_dialog = false;
        let Some(dialog) = self.delete_dialog.as_mut() else {
            return;
        };

        match key_event.code {
            KeyCode::Enter => match dialog.focused_field {
                DeleteDialogField::DeleteLocalBranch => {
                    dialog.delete_local_branch = !dialog.delete_local_branch;
                }
                DeleteDialogField::DeleteButton => {
                    confirm_delete = true;
                }
                DeleteDialogField::CancelButton => {
                    cancel_dialog = true;
                }
            },
            KeyCode::Tab => {
                dialog.focused_field = dialog.focused_field.next();
            }
            KeyCode::BackTab => {
                dialog.focused_field = dialog.focused_field.previous();
            }
            KeyCode::Up | KeyCode::Char('k') if no_modifiers => {
                dialog.focused_field = dialog.focused_field.previous();
            }
            KeyCode::Down | KeyCode::Char('j') if no_modifiers => {
                dialog.focused_field = dialog.focused_field.next();
            }
            KeyCode::Char(' ') if no_modifiers => {
                if dialog.focused_field == DeleteDialogField::DeleteLocalBranch {
                    dialog.delete_local_branch = !dialog.delete_local_branch;
                }
            }
            KeyCode::Char(character) if no_modifiers => {
                if (dialog.focused_field == DeleteDialogField::DeleteButton
                    || dialog.focused_field == DeleteDialogField::CancelButton)
                    && (character == 'h' || character == 'l')
                {
                    dialog.focused_field =
                        if dialog.focused_field == DeleteDialogField::DeleteButton {
                            DeleteDialogField::CancelButton
                        } else {
                            DeleteDialogField::DeleteButton
                        };
                }
            }
            _ => {}
        }

        if cancel_dialog {
            self.log_dialog_event("delete", "dialog_cancelled");
            self.delete_dialog = None;
            return;
        }
        if confirm_delete {
            self.confirm_delete_dialog();
        }
    }

    pub(super) fn handle_merge_dialog_key(&mut self, key_event: KeyEvent) {
        if self.merge_in_flight {
            return;
        }
        let no_modifiers = key_event.modifiers.is_empty();
        match key_event.code {
            KeyCode::Escape => {
                self.log_dialog_event("merge", "dialog_cancelled");
                self.merge_dialog = None;
                return;
            }
            KeyCode::Char('q') if no_modifiers => {
                self.log_dialog_event("merge", "dialog_cancelled");
                self.merge_dialog = None;
                return;
            }
            KeyCode::Char('m') if no_modifiers => {
                self.confirm_merge_dialog();
                return;
            }
            _ => {}
        }

        let mut confirm_merge = false;
        let mut cancel_dialog = false;
        let Some(dialog) = self.merge_dialog.as_mut() else {
            return;
        };

        match key_event.code {
            KeyCode::Enter => match dialog.focused_field {
                MergeDialogField::CleanupWorkspace => {
                    dialog.cleanup_workspace = !dialog.cleanup_workspace;
                }
                MergeDialogField::CleanupLocalBranch => {
                    dialog.cleanup_local_branch = !dialog.cleanup_local_branch;
                }
                MergeDialogField::MergeButton => {
                    confirm_merge = true;
                }
                MergeDialogField::CancelButton => {
                    cancel_dialog = true;
                }
            },
            KeyCode::Tab => {
                dialog.focused_field = dialog.focused_field.next();
            }
            KeyCode::BackTab => {
                dialog.focused_field = dialog.focused_field.previous();
            }
            KeyCode::Up | KeyCode::Char('k') if no_modifiers => {
                dialog.focused_field = dialog.focused_field.previous();
            }
            KeyCode::Down | KeyCode::Char('j') if no_modifiers => {
                dialog.focused_field = dialog.focused_field.next();
            }
            KeyCode::Char(' ') if no_modifiers => match dialog.focused_field {
                MergeDialogField::CleanupWorkspace => {
                    dialog.cleanup_workspace = !dialog.cleanup_workspace;
                }
                MergeDialogField::CleanupLocalBranch => {
                    dialog.cleanup_local_branch = !dialog.cleanup_local_branch;
                }
                MergeDialogField::MergeButton | MergeDialogField::CancelButton => {}
            },
            KeyCode::Char(character) if no_modifiers => {
                if (dialog.focused_field == MergeDialogField::MergeButton
                    || dialog.focused_field == MergeDialogField::CancelButton)
                    && (character == 'h' || character == 'l')
                {
                    dialog.focused_field = if dialog.focused_field == MergeDialogField::MergeButton
                    {
                        MergeDialogField::CancelButton
                    } else {
                        MergeDialogField::MergeButton
                    };
                }
            }
            _ => {}
        }

        if cancel_dialog {
            self.log_dialog_event("merge", "dialog_cancelled");
            self.merge_dialog = None;
            return;
        }
        if confirm_merge {
            self.confirm_merge_dialog();
        }
    }

    pub(super) fn handle_update_from_base_dialog_key(&mut self, key_event: KeyEvent) {
        if self.update_from_base_in_flight {
            return;
        }
        let no_modifiers = key_event.modifiers.is_empty();
        match key_event.code {
            KeyCode::Escape => {
                self.log_dialog_event("update_from_base", "dialog_cancelled");
                self.update_from_base_dialog = None;
                return;
            }
            KeyCode::Char('q') if no_modifiers => {
                self.log_dialog_event("update_from_base", "dialog_cancelled");
                self.update_from_base_dialog = None;
                return;
            }
            KeyCode::Char('u') if no_modifiers => {
                self.confirm_update_from_base_dialog();
                return;
            }
            _ => {}
        }

        let mut confirm_update = false;
        let mut cancel_dialog = false;
        let Some(dialog) = self.update_from_base_dialog.as_mut() else {
            return;
        };

        match key_event.code {
            KeyCode::Enter => match dialog.focused_field {
                UpdateFromBaseDialogField::UpdateButton => {
                    confirm_update = true;
                }
                UpdateFromBaseDialogField::CancelButton => {
                    cancel_dialog = true;
                }
            },
            KeyCode::Tab => {
                dialog.focused_field = dialog.focused_field.next();
            }
            KeyCode::BackTab => {
                dialog.focused_field = dialog.focused_field.previous();
            }
            KeyCode::Up | KeyCode::Char('k') if no_modifiers => {
                dialog.focused_field = dialog.focused_field.previous();
            }
            KeyCode::Down | KeyCode::Char('j') if no_modifiers => {
                dialog.focused_field = dialog.focused_field.next();
            }
            KeyCode::Char(character) if no_modifiers => {
                if (dialog.focused_field == UpdateFromBaseDialogField::UpdateButton
                    || dialog.focused_field == UpdateFromBaseDialogField::CancelButton)
                    && (character == 'h' || character == 'l')
                {
                    dialog.focused_field =
                        if dialog.focused_field == UpdateFromBaseDialogField::UpdateButton {
                            UpdateFromBaseDialogField::CancelButton
                        } else {
                            UpdateFromBaseDialogField::UpdateButton
                        };
                }
            }
            _ => {}
        }

        if cancel_dialog {
            self.log_dialog_event("update_from_base", "dialog_cancelled");
            self.update_from_base_dialog = None;
            return;
        }
        if confirm_update {
            self.confirm_update_from_base_dialog();
        }
    }

    pub(super) fn open_delete_dialog(&mut self) {
        if self.modal_open() {
            return;
        }
        if self.delete_in_flight {
            self.show_toast("workspace delete already in progress", true);
            return;
        }

        let Some(workspace) = self.state.selected_workspace() else {
            self.show_toast("no workspace selected", true);
            return;
        };
        if workspace.is_main {
            self.show_toast("cannot delete base workspace", true);
            return;
        }

        let is_missing = !workspace.path.exists();
        self.delete_dialog = Some(DeleteDialogState {
            project_name: workspace.project_name.clone(),
            project_path: workspace.project_path.clone(),
            workspace_name: workspace.name.clone(),
            branch: workspace.branch.clone(),
            path: workspace.path.clone(),
            is_missing,
            delete_local_branch: is_missing,
            focused_field: DeleteDialogField::DeleteLocalBranch,
        });
        self.log_dialog_event_with_fields(
            "delete",
            "dialog_opened",
            [
                ("workspace".to_string(), Value::from(workspace.name.clone())),
                ("branch".to_string(), Value::from(workspace.branch.clone())),
                (
                    "path".to_string(),
                    Value::from(workspace.path.display().to_string()),
                ),
                ("is_missing".to_string(), Value::from(is_missing)),
            ],
        );
        self.state.mode = UiMode::List;
        self.state.focus = PaneFocus::WorkspaceList;
        self.last_tmux_error = None;
    }

    pub(super) fn open_merge_dialog(&mut self) {
        if self.modal_open() {
            return;
        }
        if self.merge_in_flight {
            self.show_toast("workspace merge already in progress", true);
            return;
        }

        let Some(workspace) = self.state.selected_workspace() else {
            self.show_toast("no workspace selected", true);
            return;
        };
        if workspace.is_main {
            self.show_toast("cannot merge base workspace", true);
            return;
        }
        if workspace
            .base_branch
            .as_ref()
            .is_none_or(|value| value.trim().is_empty())
        {
            self.show_toast("workspace base branch marker is missing", true);
            return;
        }
        let Some(base_branch) = workspace.base_branch.clone() else {
            self.show_toast("workspace base branch marker is missing", true);
            return;
        };
        if base_branch == workspace.branch {
            self.show_toast("workspace branch already matches base branch", true);
            return;
        }

        self.merge_dialog = Some(MergeDialogState {
            project_name: workspace.project_name.clone(),
            project_path: workspace.project_path.clone(),
            workspace_name: workspace.name.clone(),
            workspace_branch: workspace.branch.clone(),
            workspace_path: workspace.path.clone(),
            base_branch,
            cleanup_workspace: true,
            cleanup_local_branch: true,
            focused_field: MergeDialogField::CleanupWorkspace,
        });
        self.log_dialog_event_with_fields(
            "merge",
            "dialog_opened",
            [
                ("workspace".to_string(), Value::from(workspace.name.clone())),
                ("branch".to_string(), Value::from(workspace.branch.clone())),
                (
                    "path".to_string(),
                    Value::from(workspace.path.display().to_string()),
                ),
                (
                    "base_branch".to_string(),
                    Value::from(
                        workspace
                            .base_branch
                            .as_ref()
                            .map_or(String::new(), ToOwned::to_owned),
                    ),
                ),
            ],
        );
        self.state.mode = UiMode::List;
        self.state.focus = PaneFocus::WorkspaceList;
        self.last_tmux_error = None;
    }

    pub(super) fn open_update_from_base_dialog(&mut self) {
        if self.modal_open() {
            return;
        }
        if self.update_from_base_in_flight {
            self.show_toast("workspace update already in progress", true);
            return;
        }

        let Some(workspace) = self.state.selected_workspace() else {
            self.show_toast("no workspace selected", true);
            return;
        };
        if workspace.is_main {
            self.show_toast("cannot update base workspace from itself", true);
            return;
        }
        if workspace
            .base_branch
            .as_ref()
            .is_none_or(|value| value.trim().is_empty())
        {
            self.show_toast("workspace base branch marker is missing", true);
            return;
        }
        let Some(base_branch) = workspace.base_branch.clone() else {
            self.show_toast("workspace base branch marker is missing", true);
            return;
        };
        if base_branch == workspace.branch {
            self.show_toast("workspace branch already matches base branch", true);
            return;
        }

        self.update_from_base_dialog = Some(UpdateFromBaseDialogState {
            project_name: workspace.project_name.clone(),
            project_path: workspace.project_path.clone(),
            workspace_name: workspace.name.clone(),
            workspace_branch: workspace.branch.clone(),
            workspace_path: workspace.path.clone(),
            base_branch,
            focused_field: UpdateFromBaseDialogField::UpdateButton,
        });
        self.log_dialog_event_with_fields(
            "update_from_base",
            "dialog_opened",
            [
                ("workspace".to_string(), Value::from(workspace.name.clone())),
                ("branch".to_string(), Value::from(workspace.branch.clone())),
                (
                    "path".to_string(),
                    Value::from(workspace.path.display().to_string()),
                ),
                (
                    "base_branch".to_string(),
                    Value::from(
                        workspace
                            .base_branch
                            .as_ref()
                            .map_or(String::new(), ToOwned::to_owned),
                    ),
                ),
            ],
        );
        self.state.mode = UiMode::List;
        self.state.focus = PaneFocus::WorkspaceList;
        self.last_tmux_error = None;
    }

    fn confirm_delete_dialog(&mut self) {
        if self.delete_in_flight {
            return;
        }

        let Some(dialog) = self.delete_dialog.take() else {
            return;
        };
        self.log_dialog_event_with_fields(
            "delete",
            "dialog_confirmed",
            [
                (
                    "workspace".to_string(),
                    Value::from(dialog.workspace_name.clone()),
                ),
                ("branch".to_string(), Value::from(dialog.branch.clone())),
                (
                    "path".to_string(),
                    Value::from(dialog.path.display().to_string()),
                ),
                (
                    "delete_local_branch".to_string(),
                    Value::from(dialog.delete_local_branch),
                ),
                ("is_missing".to_string(), Value::from(dialog.is_missing)),
            ],
        );

        let workspace_name = dialog.workspace_name.clone();
        let workspace_path = dialog.path.clone();
        let request = DeleteWorkspaceRequest {
            project_name: dialog.project_name,
            project_path: dialog.project_path,
            workspace_name: dialog.workspace_name,
            branch: dialog.branch,
            workspace_path: dialog.path,
            is_missing: dialog.is_missing,
            delete_local_branch: dialog.delete_local_branch,
        };
        if !self.tmux_input.supports_background_launch() {
            let (result, warnings) = delete_workspace(request, self.multiplexer);
            self.apply_delete_workspace_completion(DeleteWorkspaceCompletion {
                workspace_name,
                workspace_path,
                result,
                warnings,
            });
            return;
        }

        let multiplexer = self.multiplexer;
        self.delete_in_flight = true;
        self.queue_cmd(Cmd::task(move || {
            let (result, warnings) = delete_workspace(request, multiplexer);
            Msg::DeleteWorkspaceCompleted(DeleteWorkspaceCompletion {
                workspace_name,
                workspace_path,
                result,
                warnings,
            })
        }));
    }

    fn confirm_merge_dialog(&mut self) {
        if self.merge_in_flight {
            return;
        }

        let Some(dialog) = self.merge_dialog.take() else {
            return;
        };
        self.log_dialog_event_with_fields(
            "merge",
            "dialog_confirmed",
            [
                (
                    "workspace".to_string(),
                    Value::from(dialog.workspace_name.clone()),
                ),
                (
                    "workspace_branch".to_string(),
                    Value::from(dialog.workspace_branch.clone()),
                ),
                (
                    "workspace_path".to_string(),
                    Value::from(dialog.workspace_path.display().to_string()),
                ),
                (
                    "base_branch".to_string(),
                    Value::from(dialog.base_branch.clone()),
                ),
                (
                    "cleanup_workspace".to_string(),
                    Value::from(dialog.cleanup_workspace),
                ),
                (
                    "cleanup_local_branch".to_string(),
                    Value::from(dialog.cleanup_local_branch),
                ),
            ],
        );

        let workspace_name = dialog.workspace_name.clone();
        let workspace_path = dialog.workspace_path.clone();
        let workspace_branch = dialog.workspace_branch.clone();
        let base_branch = dialog.base_branch.clone();
        let request = MergeWorkspaceRequest {
            project_name: dialog.project_name,
            project_path: dialog.project_path,
            workspace_name: dialog.workspace_name,
            workspace_branch: dialog.workspace_branch,
            workspace_path: dialog.workspace_path,
            base_branch: dialog.base_branch,
            cleanup_workspace: dialog.cleanup_workspace,
            cleanup_local_branch: dialog.cleanup_local_branch,
        };

        if !self.tmux_input.supports_background_launch() {
            let (result, warnings) = merge_workspace(request, self.multiplexer);
            self.apply_merge_workspace_completion(MergeWorkspaceCompletion {
                workspace_name,
                workspace_path,
                workspace_branch,
                base_branch,
                result,
                warnings,
            });
            return;
        }

        let multiplexer = self.multiplexer;
        self.merge_in_flight = true;
        self.queue_cmd(Cmd::task(move || {
            let (result, warnings) = merge_workspace(request, multiplexer);
            Msg::MergeWorkspaceCompleted(MergeWorkspaceCompletion {
                workspace_name,
                workspace_path,
                workspace_branch,
                base_branch,
                result,
                warnings,
            })
        }));
    }

    fn confirm_update_from_base_dialog(&mut self) {
        if self.update_from_base_in_flight {
            return;
        }

        let Some(dialog) = self.update_from_base_dialog.take() else {
            return;
        };
        self.log_dialog_event_with_fields(
            "update_from_base",
            "dialog_confirmed",
            [
                (
                    "workspace".to_string(),
                    Value::from(dialog.workspace_name.clone()),
                ),
                (
                    "workspace_branch".to_string(),
                    Value::from(dialog.workspace_branch.clone()),
                ),
                (
                    "workspace_path".to_string(),
                    Value::from(dialog.workspace_path.display().to_string()),
                ),
                (
                    "base_branch".to_string(),
                    Value::from(dialog.base_branch.clone()),
                ),
            ],
        );

        let workspace_name = dialog.workspace_name.clone();
        let workspace_path = dialog.workspace_path.clone();
        let workspace_branch = dialog.workspace_branch.clone();
        let base_branch = dialog.base_branch.clone();
        let request = UpdateWorkspaceFromBaseRequest {
            project_name: dialog.project_name,
            project_path: dialog.project_path,
            workspace_name: dialog.workspace_name,
            workspace_branch: dialog.workspace_branch,
            workspace_path: dialog.workspace_path,
            base_branch: dialog.base_branch,
        };

        if !self.tmux_input.supports_background_launch() {
            let (result, warnings) = update_workspace_from_base(request, self.multiplexer);
            self.apply_update_from_base_completion(UpdateWorkspaceFromBaseCompletion {
                workspace_name,
                workspace_path,
                workspace_branch,
                base_branch,
                result,
                warnings,
            });
            return;
        }

        let multiplexer = self.multiplexer;
        self.update_from_base_in_flight = true;
        self.queue_cmd(Cmd::task(move || {
            let (result, warnings) = update_workspace_from_base(request, multiplexer);
            Msg::UpdateWorkspaceFromBaseCompleted(UpdateWorkspaceFromBaseCompletion {
                workspace_name,
                workspace_path,
                workspace_branch,
                base_branch,
                result,
                warnings,
            })
        }));
    }
}
