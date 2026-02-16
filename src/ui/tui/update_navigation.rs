use super::*;

impl GroveApp {
    fn cycle_preview_tab(&mut self, direction: i8) {
        let next_tab = if direction.is_negative() {
            self.preview_tab.previous()
        } else {
            self.preview_tab.next()
        };
        if next_tab == self.preview_tab {
            return;
        }

        self.preview_tab = next_tab;
        self.clear_preview_selection();
        if self.preview_tab == PreviewTab::Git
            && let Some(workspace) = self.state.selected_workspace()
        {
            let session_name = git_session_name_for_workspace(workspace);
            self.lazygit_failed_sessions.remove(&session_name);
        }
        self.poll_preview();
    }

    fn selected_workspace_summary(&self) -> String {
        self.state
            .selected_workspace()
            .map(|workspace| {
                if workspace.is_main && !workspace.status.has_session() {
                    return self.main_worktree_splash();
                }
                format!(
                    "Workspace: {}\nBranch: {}\nPath: {}\nAgent: {}\nOrphaned session: {}",
                    workspace.name,
                    workspace.branch,
                    workspace.path.display(),
                    workspace.agent.label(),
                    if workspace.is_orphaned { "yes" } else { "no" }
                )
            })
            .unwrap_or_else(|| "No workspace selected".to_string())
    }

    fn main_worktree_splash(&self) -> String {
        const G: &str = "\x1b[38;2;166;227;161m";
        const T: &str = "\x1b[38;2;250;179;135m";
        const R: &str = "\x1b[0m";

        [
            String::new(),
            format!("{G}                    .@@@.{R}"),
            format!("{G}                 .@@@@@@@@@.{R}"),
            format!("{G}               .@@@@@@@@@@@@@.{R}"),
            format!("{G}    .@@@.     @@@@@@@@@@@@@@@@@        .@@.{R}"),
            format!("{G}  .@@@@@@@.  @@@@@@@@@@@@@@@@@@@    .@@@@@@@@.{R}"),
            format!("{G} @@@@@@@@@@@ @@@@@@@@@@@@@@@@@@@@  @@@@@@@@@@@@@{R}"),
            format!("{G} @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@{R}"),
            format!("{G}  @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@{R}"),
            format!("{G}  '@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@'{R}"),
            format!("{G}    '@@@@@@@@  '@@@@@@@@@@@@@@@' @@@@@@@@@@@@@@'{R}"),
            format!("{G}      '@@@@'     '@@@@@@@@@@@'    '@@@@@@@@@@'{R}"),
            format!("         {T}||{R}        {G}'@@@@@@@'{R}        {G}'@@@@'{R}"),
            format!("         {T}||{R}           {T}|||{R}              {T}||{R}"),
            format!("         {T}||{R}           {T}|||{R}              {T}||{R}"),
            format!("        {T}/||\\{R}         {T}/|||\\{R}            {T}/||\\{R}"),
            String::new(),
            "Base Worktree".to_string(),
            String::new(),
            "This is your repo root.".to_string(),
            "Create focused workspaces from here when you start new work.".to_string(),
            String::new(),
            "--------------------------------------------------".to_string(),
            String::new(),
            "Press 'n' to create a workspace".to_string(),
            String::new(),
            "Each workspace has its own directory and branch.".to_string(),
            "Run agents in parallel without branch hopping.".to_string(),
        ]
        .join("\n")
    }

    fn has_non_palette_modal_open(&self) -> bool {
        self.launch_dialog.is_some()
            || self.create_dialog.is_some()
            || self.edit_dialog.is_some()
            || self.delete_dialog.is_some()
            || self.merge_dialog.is_some()
            || self.update_from_base_dialog.is_some()
            || self.settings_dialog.is_some()
            || self.project_dialog.is_some()
            || self.keybind_help_open
    }

    fn can_open_command_palette(&self) -> bool {
        !self.has_non_palette_modal_open() && self.interactive.is_none()
    }

    fn palette_action(
        id: &'static str,
        title: &'static str,
        description: &'static str,
        tags: &[&str],
        category: &'static str,
    ) -> PaletteActionItem {
        PaletteActionItem::new(id, title)
            .with_description(description)
            .with_tags(tags)
            .with_category(category)
    }

    pub(super) fn build_command_palette_actions(&self) -> Vec<PaletteActionItem> {
        let mut actions = Vec::new();
        for command in UiCommand::all() {
            if !self.palette_command_enabled(*command) {
                continue;
            }
            let Some(spec) = command.palette_spec() else {
                continue;
            };
            actions.push(Self::palette_action(
                spec.id,
                spec.title,
                spec.description,
                spec.tags,
                spec.category,
            ));
        }
        actions
    }

    fn refresh_command_palette_actions(&mut self) {
        self.command_palette
            .replace_actions(self.build_command_palette_actions());
    }

    pub(super) fn open_command_palette(&mut self) {
        if !self.can_open_command_palette() {
            return;
        }

        self.refresh_command_palette_actions();
        self.command_palette.open();
    }

    fn palette_command_enabled(&self, command: UiCommand) -> bool {
        if command.palette_spec().is_none() {
            return false;
        }
        match command {
            UiCommand::ToggleFocus
            | UiCommand::ToggleSidebar
            | UiCommand::NewWorkspace
            | UiCommand::EditWorkspace
            | UiCommand::OpenProjects
            | UiCommand::OpenSettings
            | UiCommand::ToggleUnsafe
            | UiCommand::OpenHelp
            | UiCommand::Quit => true,
            UiCommand::OpenPreview => self.state.focus == PaneFocus::WorkspaceList,
            UiCommand::EnterInteractive => {
                self.state.focus == PaneFocus::Preview
                    && workspace_can_enter_interactive(
                        self.state.selected_workspace(),
                        self.preview_tab == PreviewTab::Git,
                    )
            }
            UiCommand::FocusList => self.state.focus == PaneFocus::Preview,
            UiCommand::MoveSelectionUp | UiCommand::MoveSelectionDown => {
                self.state.focus == PaneFocus::WorkspaceList
            }
            UiCommand::ScrollUp
            | UiCommand::ScrollDown
            | UiCommand::PageUp
            | UiCommand::PageDown
            | UiCommand::ScrollBottom => self.preview_agent_tab_is_focused(),
            UiCommand::PreviousTab | UiCommand::NextTab => {
                self.state.mode == UiMode::Preview && self.state.focus == PaneFocus::Preview
            }
            UiCommand::StartAgent => {
                self.preview_agent_tab_is_focused()
                    && !self.start_in_flight
                    && workspace_can_start_agent(self.state.selected_workspace())
            }
            UiCommand::StopAgent => {
                self.preview_agent_tab_is_focused()
                    && !self.stop_in_flight
                    && workspace_can_stop_agent(self.state.selected_workspace())
            }
            UiCommand::DeleteWorkspace => {
                !self.delete_in_flight
                    && self
                        .state
                        .selected_workspace()
                        .is_some_and(|workspace| !workspace.is_main)
            }
            UiCommand::MergeWorkspace => {
                !self.merge_in_flight
                    && self
                        .state
                        .selected_workspace()
                        .is_some_and(|workspace| !workspace.is_main)
            }
            UiCommand::UpdateFromBase => {
                !self.update_from_base_in_flight
                    && self
                        .state
                        .selected_workspace()
                        .is_some_and(|workspace| !workspace.is_main)
            }
            UiCommand::FocusPreview | UiCommand::OpenCommandPalette => false,
        }
    }

    pub(super) fn execute_ui_command(&mut self, command: UiCommand) -> bool {
        match command {
            UiCommand::ToggleFocus => {
                reduce(&mut self.state, Action::ToggleFocus);
                false
            }
            UiCommand::ToggleSidebar => {
                self.sidebar_hidden = !self.sidebar_hidden;
                if self.sidebar_hidden {
                    self.divider_drag_active = false;
                }
                false
            }
            UiCommand::OpenPreview => {
                self.enter_preview_or_interactive();
                false
            }
            UiCommand::EnterInteractive => {
                self.enter_interactive(Instant::now());
                false
            }
            UiCommand::FocusPreview => {
                let mode_before = self.state.mode;
                let focus_before = self.state.focus;
                reduce(&mut self.state, Action::EnterPreviewMode);
                if self.state.mode != mode_before || self.state.focus != focus_before {
                    self.poll_preview();
                }
                false
            }
            UiCommand::FocusList => {
                reduce(&mut self.state, Action::EnterListMode);
                false
            }
            UiCommand::MoveSelectionUp => {
                self.move_selection(Action::MoveSelectionUp);
                false
            }
            UiCommand::MoveSelectionDown => {
                self.move_selection(Action::MoveSelectionDown);
                false
            }
            UiCommand::ScrollUp => {
                if self.preview_agent_tab_is_focused() {
                    self.scroll_preview(-1);
                }
                false
            }
            UiCommand::ScrollDown => {
                if self.preview_agent_tab_is_focused() {
                    self.scroll_preview(1);
                }
                false
            }
            UiCommand::PageUp => {
                if self.preview_agent_tab_is_focused() {
                    self.scroll_preview(-5);
                }
                false
            }
            UiCommand::PageDown => {
                if self.preview_agent_tab_is_focused() {
                    self.scroll_preview(5);
                }
                false
            }
            UiCommand::ScrollBottom => {
                if self.preview_agent_tab_is_focused() {
                    self.jump_preview_to_bottom();
                }
                false
            }
            UiCommand::PreviousTab => {
                if self.state.mode == UiMode::Preview && self.state.focus == PaneFocus::Preview {
                    self.cycle_preview_tab(-1);
                }
                false
            }
            UiCommand::NextTab => {
                if self.state.mode == UiMode::Preview && self.state.focus == PaneFocus::Preview {
                    self.cycle_preview_tab(1);
                }
                false
            }
            UiCommand::NewWorkspace => {
                self.open_create_dialog();
                false
            }
            UiCommand::EditWorkspace => {
                self.open_edit_dialog();
                false
            }
            UiCommand::StartAgent => {
                if self.preview_agent_tab_is_focused() {
                    self.open_start_dialog();
                }
                false
            }
            UiCommand::StopAgent => {
                if self.preview_agent_tab_is_focused() {
                    self.stop_selected_workspace_agent();
                }
                false
            }
            UiCommand::DeleteWorkspace => {
                self.open_delete_dialog();
                false
            }
            UiCommand::MergeWorkspace => {
                self.open_merge_dialog();
                false
            }
            UiCommand::UpdateFromBase => {
                self.open_update_from_base_dialog();
                false
            }
            UiCommand::OpenProjects => {
                self.open_project_dialog();
                false
            }
            UiCommand::OpenSettings => {
                self.open_settings_dialog();
                false
            }
            UiCommand::ToggleUnsafe => {
                self.launch_skip_permissions = !self.launch_skip_permissions;
                false
            }
            UiCommand::OpenHelp => {
                self.open_keybind_help();
                false
            }
            UiCommand::OpenCommandPalette => {
                self.open_command_palette();
                false
            }
            UiCommand::Quit => true,
        }
    }

    pub(super) fn execute_command_palette_action(&mut self, id: &str) -> bool {
        let Some(command) = UiCommand::from_palette_id(id) else {
            return false;
        };
        self.execute_ui_command(command)
    }

    pub(super) fn modal_open(&self) -> bool {
        self.has_non_palette_modal_open() || self.command_palette.is_visible()
    }

    pub(super) fn refresh_preview_summary(&mut self) {
        self.preview
            .apply_capture(&self.selected_workspace_summary());
    }

    pub(super) fn preview_output_dimensions(&self) -> Option<(u16, u16)> {
        let layout = self.view_layout();
        if layout.preview.is_empty() {
            return None;
        }

        let inner = Block::new().borders(Borders::ALL).inner(layout.preview);
        if inner.is_empty() || inner.width == 0 {
            return None;
        }

        let output_height = inner.height.saturating_sub(PREVIEW_METADATA_ROWS).max(1);
        Some((inner.width, output_height))
    }

    fn ensure_lazygit_session_for_selected_workspace(&mut self) -> Option<String> {
        let workspace = self.state.selected_workspace()?;
        let session_name = git_session_name_for_workspace(workspace);

        if self.lazygit_ready_sessions.contains(&session_name) {
            return Some(session_name);
        }
        if self.lazygit_failed_sessions.contains(&session_name) {
            return None;
        }
        if self.lazygit_launch_in_flight.contains(&session_name) {
            return None;
        }

        let capture_cols = self
            .preview_output_dimensions()
            .map_or(self.viewport_width.saturating_sub(4), |(width, _)| width)
            .max(80);
        let capture_rows = self.viewport_height.saturating_sub(4).max(1);
        let launch_request = shell_launch_request_for_workspace(
            workspace,
            session_name.clone(),
            LAZYGIT_COMMAND.to_string(),
            Some(capture_cols),
            Some(capture_rows),
        );
        let async_launch = self.tmux_input.supports_background_launch();
        self.event_log.log(
            LogEvent::new("lazygit_launch", "started")
                .with_data("session", Value::from(session_name.clone()))
                .with_data("multiplexer", Value::from(self.multiplexer.label()))
                .with_data("async", Value::from(async_launch))
                .with_data("capture_cols", Value::from(capture_cols))
                .with_data("capture_rows", Value::from(capture_rows)),
        );

        if async_launch {
            self.lazygit_launch_in_flight.insert(session_name.clone());
            let multiplexer = self.multiplexer;
            let completion_session = session_name.clone();
            self.queue_cmd(Cmd::task(move || {
                let started_at = Instant::now();
                let (_, result) = execute_shell_launch_request_for_mode(
                    &launch_request,
                    multiplexer,
                    CommandExecutionMode::Process,
                );
                let duration_ms =
                    GroveApp::duration_millis(Instant::now().saturating_duration_since(started_at));
                Msg::LazygitLaunchCompleted(LazygitLaunchCompletion {
                    session_name: completion_session,
                    duration_ms,
                    result,
                })
            }));
            return None;
        }

        let launch_started_at = Instant::now();
        let (_, launch_result) = execute_shell_launch_request_for_mode(
            &launch_request,
            self.multiplexer,
            CommandExecutionMode::Delegating(&mut |command| self.execute_tmux_command(command)),
        );
        let duration_ms =
            Self::duration_millis(Instant::now().saturating_duration_since(launch_started_at));
        let mut completion_event = LogEvent::new("lazygit_launch", "completed")
            .with_data("session", Value::from(session_name.clone()))
            .with_data("multiplexer", Value::from(self.multiplexer.label()))
            .with_data("async", Value::from(false))
            .with_data("duration_ms", Value::from(duration_ms))
            .with_data("ok", Value::from(launch_result.is_ok()));

        if let Err(error) = launch_result {
            completion_event = completion_event.with_data("error", Value::from(error.clone()));
            self.event_log.log(completion_event);
            self.last_tmux_error = Some(error);
            self.show_toast("lazygit launch failed", true);
            self.lazygit_ready_sessions.remove(&session_name);
            self.lazygit_failed_sessions.insert(session_name);
            return None;
        }

        self.event_log.log(completion_event);
        self.lazygit_failed_sessions.remove(&session_name);
        self.lazygit_ready_sessions.insert(session_name.clone());
        Some(session_name)
    }

    pub(super) fn prepare_live_preview_session(&mut self) -> Option<LivePreviewTarget> {
        if self.preview_tab == PreviewTab::Git {
            return self
                .ensure_lazygit_session_for_selected_workspace()
                .map(|session_name| LivePreviewTarget {
                    session_name,
                    include_escape_sequences: true,
                });
        }
        live_preview_capture_target_for_tab(
            self.state.selected_workspace(),
            false,
            &self.lazygit_ready_sessions,
        )
    }

    pub(super) fn handle_lazygit_launch_completed(&mut self, completion: LazygitLaunchCompletion) {
        let LazygitLaunchCompletion {
            session_name,
            duration_ms,
            result,
        } = completion;
        self.lazygit_launch_in_flight.remove(&session_name);

        let mut completion_event = LogEvent::new("lazygit_launch", "completed")
            .with_data("session", Value::from(session_name.clone()))
            .with_data("multiplexer", Value::from(self.multiplexer.label()))
            .with_data("async", Value::from(true))
            .with_data("duration_ms", Value::from(duration_ms))
            .with_data("ok", Value::from(result.is_ok()));

        match result {
            Ok(()) => {
                self.last_tmux_error = None;
                self.lazygit_failed_sessions.remove(&session_name);
                self.lazygit_ready_sessions.insert(session_name.clone());
                self.event_log.log(completion_event);

                let selected_session_matches =
                    self.state.selected_workspace().is_some_and(|workspace| {
                        git_session_name_for_workspace(workspace) == session_name
                    });
                if selected_session_matches && self.preview_tab == PreviewTab::Git {
                    self.poll_preview();
                }
            }
            Err(error) => {
                completion_event = completion_event.with_data("error", Value::from(error.clone()));
                self.event_log.log(completion_event);
                self.last_tmux_error = Some(error.clone());
                self.log_tmux_error(error);
                self.lazygit_ready_sessions.remove(&session_name);
                self.lazygit_failed_sessions.insert(session_name);
                self.show_toast("lazygit launch failed", true);
            }
        }
    }

    pub(super) fn interactive_target_session(&self) -> Option<String> {
        self.interactive
            .as_ref()
            .map(|state| state.target_session.clone())
    }
}
