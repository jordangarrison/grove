use super::*;

impl GroveApp {
    fn poll_preview_sync(&mut self) {
        let live_preview = self.prepare_live_preview_session();
        let has_live_preview = live_preview.is_some();
        let cursor_session = self.interactive_target_session();
        let status_poll_targets = workspace_status_targets_for_polling_with_live_preview(
            &self.state.workspaces,
            self.multiplexer,
            live_preview.as_ref(),
        );

        if let Some(live_preview_target) = live_preview {
            let capture_started_at = Instant::now();
            let result = self
                .tmux_input
                .capture_output(
                    &live_preview_target.session_name,
                    600,
                    live_preview_target.include_escape_sequences,
                )
                .map_err(|error| error.to_string());
            let capture_ms =
                Self::duration_millis(Instant::now().saturating_duration_since(capture_started_at));
            self.apply_live_preview_capture(
                &live_preview_target.session_name,
                live_preview_target.include_escape_sequences,
                capture_ms,
                capture_ms,
                result,
            );
        } else {
            self.clear_agent_activity_tracking();
            self.refresh_preview_summary();
        }

        for target in status_poll_targets {
            let capture_started_at = Instant::now();
            let result = self
                .tmux_input
                .capture_output(&target.session_name, 120, false)
                .map_err(|error| error.to_string());
            let capture_ms =
                Self::duration_millis(Instant::now().saturating_duration_since(capture_started_at));
            self.apply_workspace_status_capture(WorkspaceStatusCapture {
                workspace_name: target.workspace_name,
                workspace_path: target.workspace_path,
                session_name: target.session_name,
                supported_agent: target.supported_agent,
                capture_ms,
                result,
            });
        }
        if !has_live_preview {
            self.refresh_preview_summary();
        }

        if let Some(target_session) = cursor_session {
            self.poll_interactive_cursor_sync(&target_session);
        }
    }

    fn schedule_async_preview_poll(
        &self,
        generation: u64,
        live_preview: Option<LivePreviewTarget>,
        cursor_session: Option<String>,
        status_poll_targets: Vec<WorkspaceStatusPollTarget>,
    ) -> Cmd<Msg> {
        Cmd::task(move || {
            let live_capture = live_preview.map(|target| {
                let capture_started_at = Instant::now();
                let result = CommandTmuxInput::capture_session_output(
                    &target.session_name,
                    600,
                    target.include_escape_sequences,
                )
                .map_err(|error| error.to_string());
                let capture_ms = GroveApp::duration_millis(
                    Instant::now().saturating_duration_since(capture_started_at),
                );
                LivePreviewCapture {
                    session: target.session_name,
                    include_escape_sequences: target.include_escape_sequences,
                    capture_ms,
                    total_ms: capture_ms,
                    result,
                }
            });

            let cursor_capture = cursor_session.map(|session| {
                let started_at = Instant::now();
                let result = CommandTmuxInput::capture_session_cursor_metadata(&session)
                    .map_err(|error| error.to_string());
                let capture_ms =
                    GroveApp::duration_millis(Instant::now().saturating_duration_since(started_at));
                CursorCapture {
                    session,
                    capture_ms,
                    result,
                }
            });

            let workspace_status_captures = status_poll_targets
                .into_iter()
                .map(|target| {
                    let capture_started_at = Instant::now();
                    let result =
                        CommandTmuxInput::capture_session_output(&target.session_name, 120, false)
                            .map_err(|error| error.to_string());
                    let capture_ms = GroveApp::duration_millis(
                        Instant::now().saturating_duration_since(capture_started_at),
                    );
                    WorkspaceStatusCapture {
                        workspace_name: target.workspace_name,
                        workspace_path: target.workspace_path,
                        session_name: target.session_name,
                        supported_agent: target.supported_agent,
                        capture_ms,
                        result,
                    }
                })
                .collect();

            Msg::PreviewPollCompleted(PreviewPollCompletion {
                generation,
                live_capture,
                cursor_capture,
                workspace_status_captures,
            })
        })
    }

    pub(super) fn poll_preview(&mut self) {
        if !self.tmux_input.supports_background_poll() {
            self.poll_preview_sync();
            return;
        }
        if self.preview_poll_in_flight {
            self.preview_poll_requested = true;
            return;
        }

        let live_preview = self.prepare_live_preview_session();
        let cursor_session = self.interactive_target_session();
        let status_poll_targets = workspace_status_targets_for_polling_with_live_preview(
            &self.state.workspaces,
            self.multiplexer,
            live_preview.as_ref(),
        );

        if live_preview.is_none() && cursor_session.is_none() && status_poll_targets.is_empty() {
            self.preview_poll_requested = false;
            self.clear_agent_activity_tracking();
            self.refresh_preview_summary();
            return;
        }

        self.poll_generation = self.poll_generation.saturating_add(1);
        self.preview_poll_in_flight = true;
        self.preview_poll_requested = false;
        self.queue_cmd(self.schedule_async_preview_poll(
            self.poll_generation,
            live_preview,
            cursor_session,
            status_poll_targets,
        ));
    }

    pub(super) fn handle_preview_poll_completed(&mut self, completion: PreviewPollCompletion) {
        if completion.generation < self.poll_generation {
            self.event_log.log(
                LogEvent::new("preview_poll", "stale_result_dropped")
                    .with_data("generation", Value::from(completion.generation))
                    .with_data("latest_generation", Value::from(self.poll_generation)),
            );
            return;
        }

        self.preview_poll_in_flight = false;
        if completion.generation > self.poll_generation {
            self.poll_generation = completion.generation;
        }

        let had_live_capture = completion.live_capture.is_some();
        if let Some(live_capture) = completion.live_capture {
            self.apply_live_preview_capture(
                &live_capture.session,
                live_capture.include_escape_sequences,
                live_capture.capture_ms,
                live_capture.total_ms,
                live_capture.result,
            );
        } else {
            self.clear_agent_activity_tracking();
            self.refresh_preview_summary();
        }

        for status_capture in completion.workspace_status_captures {
            self.apply_workspace_status_capture(status_capture);
        }
        if !had_live_capture {
            self.refresh_preview_summary();
        }

        if let Some(cursor_capture) = completion.cursor_capture {
            self.apply_cursor_capture_result(cursor_capture);
        }

        if self.preview_poll_requested {
            self.preview_poll_requested = false;
            self.poll_preview();
        }
    }
}
