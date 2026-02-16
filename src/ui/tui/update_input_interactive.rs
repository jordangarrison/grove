use super::*;

impl GroveApp {
    fn map_interactive_key(key_event: KeyEvent) -> Option<InteractiveKey> {
        let ctrl = key_event.modifiers.contains(Modifiers::CTRL);
        let alt = key_event.modifiers.contains(Modifiers::ALT);

        match key_event.code {
            KeyCode::Enter => Some(InteractiveKey::Enter),
            KeyCode::Tab => Some(InteractiveKey::Tab),
            KeyCode::Backspace => Some(InteractiveKey::Backspace),
            KeyCode::Delete => Some(InteractiveKey::Delete),
            KeyCode::Up => Some(InteractiveKey::Up),
            KeyCode::Down => Some(InteractiveKey::Down),
            KeyCode::Left => Some(InteractiveKey::Left),
            KeyCode::Right => Some(InteractiveKey::Right),
            KeyCode::Home => Some(InteractiveKey::Home),
            KeyCode::End => Some(InteractiveKey::End),
            KeyCode::PageUp => Some(InteractiveKey::PageUp),
            KeyCode::PageDown => Some(InteractiveKey::PageDown),
            KeyCode::Escape => Some(InteractiveKey::Escape),
            KeyCode::F(index) => Some(InteractiveKey::Function(index)),
            KeyCode::Char(character) => {
                if (ctrl && matches!(character, '\\' | '|' | '4')) || character == '\u{1c}' {
                    return Some(InteractiveKey::CtrlBackslash);
                }
                if alt && matches!(character, 'c' | 'C') {
                    return Some(InteractiveKey::AltC);
                }
                if alt && matches!(character, 'v' | 'V') {
                    return Some(InteractiveKey::AltV);
                }
                if ctrl {
                    return Some(InteractiveKey::Ctrl(character));
                }
                Some(InteractiveKey::Char(character))
            }
            _ => None,
        }
    }

    fn queue_interactive_send(&mut self, send: QueuedInteractiveSend) -> Cmd<Msg> {
        self.pending_interactive_sends.push_back(send);
        self.dispatch_next_interactive_send()
    }

    fn dispatch_next_interactive_send(&mut self) -> Cmd<Msg> {
        if self.interactive_send_in_flight {
            return Cmd::None;
        }
        let Some(send) = self.pending_interactive_sends.pop_front() else {
            return Cmd::None;
        };
        self.interactive_send_in_flight = true;
        let command = send.command.clone();
        Cmd::task(move || {
            let started_at = Instant::now();
            let execution = CommandTmuxInput::execute_command(&command);
            let completed_at = Instant::now();
            let tmux_send_ms = u64::try_from(
                completed_at
                    .saturating_duration_since(started_at)
                    .as_millis(),
            )
            .unwrap_or(u64::MAX);
            Msg::InteractiveSendCompleted(InteractiveSendCompletion {
                send,
                tmux_send_ms,
                error: execution.err().map(|error| error.to_string()),
            })
        })
    }

    pub(super) fn handle_interactive_send_completed(
        &mut self,
        completion: InteractiveSendCompletion,
    ) -> Cmd<Msg> {
        let InteractiveSendCompletion {
            send:
                QueuedInteractiveSend {
                    target_session,
                    action_kind,
                    trace_context,
                    literal_chars,
                    ..
                },
            tmux_send_ms,
            error,
        } = completion;
        self.interactive_send_in_flight = false;
        if let Some(error) = error {
            self.last_tmux_error = Some(error.clone());
            self.log_tmux_error(error.clone());
            if let Some(trace_context) = trace_context {
                self.log_input_event_with_fields(
                    "interactive_forward_failed",
                    trace_context.seq,
                    vec![
                        ("session".to_string(), Value::from(target_session)),
                        ("action".to_string(), Value::from(action_kind)),
                        ("error".to_string(), Value::from(error)),
                    ],
                );
            }
            return self.dispatch_next_interactive_send();
        }

        self.last_tmux_error = None;
        if let Some(trace_context) = trace_context {
            let forwarded_at = Instant::now();
            self.track_pending_interactive_input(trace_context, &target_session, forwarded_at);
            let mut fields = vec![
                ("session".to_string(), Value::from(target_session)),
                ("action".to_string(), Value::from(action_kind)),
                ("tmux_send_ms".to_string(), Value::from(tmux_send_ms)),
                (
                    "queue_depth".to_string(),
                    Value::from(
                        u64::try_from(self.pending_interactive_inputs.len()).unwrap_or(u64::MAX),
                    ),
                ),
            ];
            if let Some(literal_chars) = literal_chars {
                fields.push(("literal_chars".to_string(), Value::from(literal_chars)));
            }
            self.log_input_event_with_fields("interactive_forwarded", trace_context.seq, fields);
        }
        self.dispatch_next_interactive_send()
    }

    pub(super) fn send_interactive_action(
        &mut self,
        action: &InteractiveAction,
        target_session: &str,
        trace_context: Option<InputTraceContext>,
    ) -> Cmd<Msg> {
        let Some(command) =
            multiplexer_send_input_command(self.multiplexer, target_session, action)
        else {
            if let Some(trace_context) = trace_context {
                self.log_input_event_with_fields(
                    "interactive_action_unmapped",
                    trace_context.seq,
                    vec![
                        (
                            "action".to_string(),
                            Value::from(Self::interactive_action_kind(action)),
                        ),
                        (
                            "session".to_string(),
                            Value::from(target_session.to_string()),
                        ),
                    ],
                );
            }
            return Cmd::None;
        };

        let literal_chars = if let InteractiveAction::SendLiteral(text) = action {
            Some(u64::try_from(text.chars().count()).unwrap_or(u64::MAX))
        } else {
            None
        };

        if self.tmux_input.supports_background_send() {
            return self.queue_interactive_send(QueuedInteractiveSend {
                command,
                target_session: target_session.to_string(),
                action_kind: Self::interactive_action_kind(action).to_string(),
                trace_context,
                literal_chars,
            });
        }

        let send_started_at = Instant::now();
        match self.execute_tmux_command(&command) {
            Ok(()) => {
                self.last_tmux_error = None;
                if let Some(trace_context) = trace_context {
                    let forwarded_at = Instant::now();
                    let send_duration_ms = Self::duration_millis(
                        forwarded_at.saturating_duration_since(send_started_at),
                    );
                    self.track_pending_interactive_input(
                        trace_context,
                        target_session,
                        forwarded_at,
                    );

                    let mut fields = vec![
                        (
                            "session".to_string(),
                            Value::from(target_session.to_string()),
                        ),
                        (
                            "action".to_string(),
                            Value::from(Self::interactive_action_kind(action)),
                        ),
                        ("tmux_send_ms".to_string(), Value::from(send_duration_ms)),
                        (
                            "queue_depth".to_string(),
                            Value::from(
                                u64::try_from(self.pending_interactive_inputs.len())
                                    .unwrap_or(u64::MAX),
                            ),
                        ),
                    ];
                    if let Some(literal_chars) = literal_chars {
                        fields.push(("literal_chars".to_string(), Value::from(literal_chars)));
                    }
                    self.log_input_event_with_fields(
                        "interactive_forwarded",
                        trace_context.seq,
                        fields,
                    );
                }
            }
            Err(error) => {
                let message = error.to_string();
                self.last_tmux_error = Some(message.clone());
                self.log_tmux_error(message);
                if let Some(trace_context) = trace_context {
                    self.log_input_event_with_fields(
                        "interactive_forward_failed",
                        trace_context.seq,
                        vec![
                            (
                                "session".to_string(),
                                Value::from(target_session.to_string()),
                            ),
                            (
                                "action".to_string(),
                                Value::from(Self::interactive_action_kind(action)),
                            ),
                            ("error".to_string(), Value::from(error.to_string())),
                        ],
                    );
                }
            }
        }
        Cmd::None
    }

    fn copy_interactive_capture(&mut self) {
        self.copy_interactive_selection_or_visible();
    }

    fn read_clipboard_or_cached_text(&mut self) -> Result<String, String> {
        let clipboard_text = self.clipboard.read_text();
        if let Ok(text) = clipboard_text
            && !text.is_empty()
        {
            return Ok(text);
        }

        if let Some(text) = self.copied_text.clone()
            && !text.is_empty()
        {
            return Ok(text);
        }

        Err("clipboard empty".to_string())
    }

    fn paste_clipboard_text(
        &mut self,
        target_session: &str,
        bracketed_paste: bool,
        trace_context: Option<InputTraceContext>,
    ) -> Cmd<Msg> {
        let text = match self.read_clipboard_or_cached_text() {
            Ok(text) => text,
            Err(error) => {
                self.last_tmux_error = Some(error.clone());
                if let Some(trace_context) = trace_context {
                    self.log_input_event_with_fields(
                        "paste_clipboard_missing",
                        trace_context.seq,
                        vec![(
                            "session".to_string(),
                            Value::from(target_session.to_string()),
                        )],
                    );
                }
                return Cmd::None;
            }
        };

        if bracketed_paste {
            let payload = format!("\u{1b}[200~{text}\u{1b}[201~");
            return self.send_interactive_action(
                &InteractiveAction::SendLiteral(payload),
                target_session,
                trace_context,
            );
        }

        match self.tmux_input.paste_buffer(target_session, &text) {
            Ok(()) => {
                self.last_tmux_error = None;
            }
            Err(error) => {
                let message = error.to_string();
                self.last_tmux_error = Some(message.clone());
                self.log_tmux_error(message.clone());
                if let Some(trace_context) = trace_context {
                    self.log_input_event_with_fields(
                        "interactive_paste_buffer_failed",
                        trace_context.seq,
                        vec![
                            (
                                "session".to_string(),
                                Value::from(target_session.to_string()),
                            ),
                            ("error".to_string(), Value::from(message)),
                        ],
                    );
                }
            }
        }

        Cmd::None
    }

    pub(super) fn handle_interactive_key(&mut self, key_event: KeyEvent) -> Cmd<Msg> {
        let now = Instant::now();
        let input_seq = self.next_input_seq();
        if let KeyCode::Char(character) = key_event.code
            && key_event.modifiers.is_empty()
            && let Some(state) = self.interactive.as_mut()
            && state.should_drop_split_mouse_fragment(character, now)
        {
            self.log_input_event_with_fields(
                "interactive_key_dropped_mouse_fragment",
                input_seq,
                vec![
                    ("code".to_string(), Value::from("char")),
                    ("modifiers".to_string(), Value::from("none")),
                ],
            );
            return Cmd::None;
        }

        let Some(interactive_key) = Self::map_interactive_key(key_event) else {
            self.log_input_event_with_fields(
                "interactive_key_unmapped",
                input_seq,
                vec![(
                    "code".to_string(),
                    Value::from(format!("{:?}", key_event.code)),
                )],
            );
            return Cmd::None;
        };
        self.log_input_event_with_fields(
            "interactive_key_received",
            input_seq,
            vec![
                (
                    "key".to_string(),
                    Value::from(Self::interactive_key_kind(&interactive_key)),
                ),
                (
                    "repeat".to_string(),
                    Value::from(matches!(key_event.kind, KeyEventKind::Repeat)),
                ),
            ],
        );

        let (action, target_session, bracketed_paste) = {
            let Some(state) = self.interactive.as_mut() else {
                return Cmd::None;
            };
            let action = state.handle_key(interactive_key, now);
            let session = state.target_session.clone();
            let bracketed_paste = state.bracketed_paste;
            (action, session, bracketed_paste)
        };
        self.log_input_event_with_fields(
            "interactive_action_selected",
            input_seq,
            vec![
                (
                    "action".to_string(),
                    Value::from(Self::interactive_action_kind(&action)),
                ),
                ("session".to_string(), Value::from(target_session.clone())),
            ],
        );
        let trace_context = InputTraceContext {
            seq: input_seq,
            received_at: now,
        };

        match action {
            InteractiveAction::ExitInteractive => {
                self.interactive = None;
                self.state.mode = UiMode::Preview;
                self.state.focus = PaneFocus::Preview;
                self.clear_preview_selection();
                Cmd::None
            }
            InteractiveAction::CopySelection => {
                self.copy_interactive_capture();
                Cmd::None
            }
            InteractiveAction::PasteClipboard => {
                if self.preview.offset > 0 {
                    self.preview.jump_to_bottom();
                }
                let send_cmd = self.paste_clipboard_text(
                    &target_session,
                    bracketed_paste,
                    Some(trace_context),
                );
                self.schedule_interactive_debounced_poll(now);
                send_cmd
            }
            InteractiveAction::Noop
            | InteractiveAction::SendNamed(_)
            | InteractiveAction::SendLiteral(_) => {
                let send_cmd =
                    self.send_interactive_action(&action, &target_session, Some(trace_context));
                self.schedule_interactive_debounced_poll(now);
                send_cmd
            }
        }
    }

    pub(super) fn is_ctrl_char_key(key_event: &KeyEvent, character: char) -> bool {
        matches!(
            key_event.code,
            KeyCode::Char(value)
                if value == character
                    && key_event.kind == KeyEventKind::Press
                    && key_event.modifiers == Modifiers::CTRL
        )
    }

    pub(super) fn enter_interactive(&mut self, now: Instant) -> bool {
        if !workspace_can_enter_interactive(
            self.state.selected_workspace(),
            self.preview_tab == PreviewTab::Git,
        ) {
            return false;
        }

        let git_preview_session = if self.preview_tab == PreviewTab::Git {
            let Some(target) = self.prepare_live_preview_session() else {
                return false;
            };
            Some(target.session_name)
        } else {
            None
        };

        let Some(session_name) = workspace_session_for_preview_tab(
            self.state.selected_workspace(),
            self.preview_tab == PreviewTab::Git,
            git_preview_session.as_deref(),
        ) else {
            return false;
        };

        self.interactive = Some(InteractiveState::new(
            "%0".to_string(),
            session_name,
            now,
            self.viewport_height,
            self.viewport_width,
        ));
        self.interactive_poll_due_at = None;
        self.last_tmux_error = None;
        self.state.mode = UiMode::Preview;
        self.state.focus = PaneFocus::Preview;
        self.clear_preview_selection();
        self.sync_interactive_session_geometry();
        self.poll_preview();
        true
    }
}
