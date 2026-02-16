use super::*;

impl GroveApp {
    fn frame_lines_hash(lines: &[String]) -> u64 {
        let mut hasher = DefaultHasher::new();
        lines.hash(&mut hasher);
        hasher.finish()
    }

    fn frame_buffer_lines(frame: &mut Frame) -> Vec<String> {
        let height = frame.buffer.height();
        let mut lines = Vec::with_capacity(usize::from(height));
        for y in 0..height {
            let mut row = String::with_capacity(usize::from(frame.buffer.width()));
            for x in 0..frame.buffer.width() {
                let Some(cell) = frame.buffer.get(x, y).copied() else {
                    continue;
                };
                if cell.is_continuation() {
                    continue;
                }
                if let Some(value) = cell.content.as_char() {
                    row.push(value);
                    continue;
                }
                if let Some(grapheme_id) = cell.content.grapheme_id()
                    && let Some(grapheme) = frame.pool.get(grapheme_id)
                {
                    row.push_str(grapheme);
                    continue;
                }
                row.push(' ');
            }
            lines.push(row.trim_end_matches(' ').to_string());
        }

        lines
    }

    pub(super) fn log_frame_render(&self, frame: &mut Frame) {
        let Some(app_start_ts) = self.debug_record_start_ts else {
            return;
        };

        let lines = Self::frame_buffer_lines(frame);
        let frame_hash = Self::frame_lines_hash(&lines);
        let non_empty_line_count = lines.iter().filter(|line| !line.is_empty()).count();
        let mut seq = self.frame_render_seq.borrow_mut();
        *seq = seq.saturating_add(1);
        let seq_value = *seq;

        let selected_workspace = self
            .state
            .selected_workspace()
            .map(|workspace| workspace.name.clone())
            .unwrap_or_default();
        let interactive_session = self
            .interactive
            .as_ref()
            .map(|state| state.target_session.clone())
            .unwrap_or_default();
        let pending_input_depth = self.pending_input_depth();
        let oldest_pending_input_seq = self
            .pending_interactive_inputs
            .front()
            .map(|trace| trace.seq)
            .unwrap_or(0);
        let oldest_pending_input_age_ms = self.oldest_pending_input_age_ms(Instant::now());

        let mut frame_event = LogEvent::new("frame", "rendered")
            .with_data("seq", Value::from(seq_value))
            .with_data("app_start_ts", Value::from(app_start_ts))
            .with_data("width", Value::from(frame.buffer.width()))
            .with_data("height", Value::from(frame.buffer.height()))
            .with_data(
                "line_count",
                Value::from(u64::try_from(lines.len()).unwrap_or(u64::MAX)),
            )
            .with_data(
                "non_empty_line_count",
                Value::from(u64::try_from(non_empty_line_count).unwrap_or(u64::MAX)),
            )
            .with_data("frame_hash", Value::from(frame_hash))
            .with_data("degradation", Value::from(frame.degradation.as_str()))
            .with_data("mode", Value::from(self.mode_label()))
            .with_data("focus", Value::from(self.focus_label()))
            .with_data("selected_workspace", Value::from(selected_workspace))
            .with_data("interactive_session", Value::from(interactive_session))
            .with_data("sidebar_width_pct", Value::from(self.sidebar_width_pct))
            .with_data(
                "preview_offset",
                Value::from(u64::try_from(self.preview.offset).unwrap_or(u64::MAX)),
            )
            .with_data("preview_auto_scroll", Value::from(self.preview.auto_scroll))
            .with_data("output_changing", Value::from(self.output_changing))
            .with_data("pending_input_depth", Value::from(pending_input_depth))
            .with_data(
                "oldest_pending_input_seq",
                Value::from(oldest_pending_input_seq),
            )
            .with_data(
                "oldest_pending_input_age_ms",
                Value::from(oldest_pending_input_age_ms),
            )
            .with_data("frame_cursor_visible", Value::from(frame.cursor_visible))
            .with_data(
                "frame_cursor_has_position",
                Value::from(frame.cursor_position.is_some()),
            );
        if let Some((cursor_col, cursor_row)) = frame.cursor_position {
            frame_event = frame_event
                .with_data("frame_cursor_col", Value::from(cursor_col))
                .with_data("frame_cursor_row", Value::from(cursor_row));
        }
        if let Some(interactive) = self.interactive.as_ref() {
            frame_event = frame_event
                .with_data(
                    "interactive_cursor_visible",
                    Value::from(interactive.cursor_visible),
                )
                .with_data(
                    "interactive_cursor_row",
                    Value::from(interactive.cursor_row),
                )
                .with_data(
                    "interactive_cursor_col",
                    Value::from(interactive.cursor_col),
                )
                .with_data(
                    "interactive_pane_width",
                    Value::from(interactive.pane_width),
                )
                .with_data(
                    "interactive_pane_height",
                    Value::from(interactive.pane_height),
                );

            let layout = Self::view_layout_for_size(
                frame.buffer.width(),
                frame.buffer.height(),
                self.sidebar_width_pct,
            );
            let preview_inner = Block::new().borders(Borders::ALL).inner(layout.preview);
            let preview_height = usize::from(
                preview_inner
                    .height
                    .saturating_sub(PREVIEW_METADATA_ROWS)
                    .max(1),
            );
            let cursor_target = self.interactive_cursor_target(preview_height);
            frame_event = frame_event.with_data(
                "interactive_cursor_in_viewport",
                Value::from(cursor_target.is_some()),
            );
            if let Some((visible_index, target_col, target_visible)) = cursor_target {
                frame_event = frame_event
                    .with_data(
                        "interactive_cursor_visible_index",
                        Value::from(u64::try_from(visible_index).unwrap_or(u64::MAX)),
                    )
                    .with_data(
                        "interactive_cursor_target_col",
                        Value::from(u64::try_from(target_col).unwrap_or(u64::MAX)),
                    )
                    .with_data(
                        "interactive_cursor_target_visible",
                        Value::from(target_visible),
                    );
            }
        }
        frame_event = frame_event.with_data(
            "frame_lines",
            Value::Array(lines.into_iter().map(Value::from).collect()),
        );
        self.event_log.log(frame_event);
    }
}
