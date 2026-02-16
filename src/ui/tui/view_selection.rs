use super::*;

impl GroveApp {
    pub(super) fn clear_preview_selection(&mut self) {
        self.preview_selection.clear();
    }

    pub(super) fn preview_visible_range_for_height(&self, preview_height: usize) -> (usize, usize) {
        if preview_height == 0 {
            return (0, 0);
        }

        let max_offset = self.preview.max_scroll_offset(preview_height);
        let clamped_offset = self.preview.offset.min(max_offset);
        let end = self.preview.lines.len().saturating_sub(clamped_offset);
        let start = end.saturating_sub(preview_height);
        (start, end)
    }

    fn preview_content_viewport(&self) -> Option<PreviewContentViewport> {
        let layout = self.view_layout();
        if layout.preview.is_empty() {
            return None;
        }
        let inner = Block::new().borders(Borders::ALL).inner(layout.preview);
        if inner.is_empty() {
            return None;
        }

        let preview_height = usize::from(inner.height)
            .saturating_sub(usize::from(PREVIEW_METADATA_ROWS))
            .max(1);
        let (visible_start, visible_end) = self.preview_visible_range_for_height(preview_height);

        Some(PreviewContentViewport {
            output_x: inner.x,
            output_y: inner.y.saturating_add(PREVIEW_METADATA_ROWS),
            visible_start,
            visible_end,
        })
    }

    pub(super) fn preview_text_point_at(&self, x: u16, y: u16) -> Option<TextSelectionPoint> {
        if self.preview_tab != PreviewTab::Agent {
            return None;
        }

        let viewport = self.preview_content_viewport()?;
        if y < viewport.output_y {
            return None;
        }

        let visible_row = usize::from(y - viewport.output_y);
        let visible_count = viewport.visible_end.saturating_sub(viewport.visible_start);
        if visible_row >= visible_count {
            return None;
        }

        let line_idx = viewport.visible_start.saturating_add(visible_row);
        let line = self.preview_plain_line(line_idx)?;
        let line_width = line_visual_width(&line);
        if x < viewport.output_x {
            return Some(TextSelectionPoint {
                line: line_idx,
                col: 0,
            });
        }

        let relative_x = usize::from(x - viewport.output_x);
        let col = if line_width == 0 {
            0
        } else {
            relative_x.min(line_width.saturating_sub(1))
        };

        Some(TextSelectionPoint {
            line: line_idx,
            col,
        })
    }

    fn preview_plain_line(&self, line_idx: usize) -> Option<String> {
        if let Some(line) = self.preview.render_lines.get(line_idx) {
            return Some(ansi_line_to_plain_text(line));
        }

        self.preview.lines.get(line_idx).cloned()
    }

    pub(super) fn preview_plain_lines_range(&self, start: usize, end: usize) -> Vec<String> {
        if start >= end {
            return Vec::new();
        }

        let mut lines = Vec::with_capacity(end.saturating_sub(start));
        for line_idx in start..end {
            if let Some(line) = self.preview_plain_line(line_idx) {
                lines.push(line);
                continue;
            }
            break;
        }

        lines
    }

    pub(super) fn add_selection_point_snapshot_fields(
        &self,
        mut event: LogEvent,
        key_prefix: &str,
        point: TextSelectionPoint,
    ) -> LogEvent {
        let raw_line = self.preview.lines.get(point.line).cloned();
        let clean_line = self.preview_plain_line(point.line);
        let render_line = self.preview.render_lines.get(point.line).cloned();

        if let Some(line) = raw_line {
            event = event.with_data(
                format!("{key_prefix}line_raw_preview"),
                Value::from(truncate_for_log(&line, 120)),
            );
        }

        if let Some(line) = clean_line {
            event = event
                .with_data(
                    format!("{key_prefix}line_clean_preview"),
                    Value::from(truncate_for_log(&line, 120)),
                )
                .with_data(
                    format!("{key_prefix}line_visual_width"),
                    Value::from(u64::try_from(line_visual_width(&line)).unwrap_or(u64::MAX)),
                )
                .with_data(
                    format!("{key_prefix}line_context"),
                    Value::from(truncate_for_log(
                        &visual_substring(
                            &line,
                            point.col.saturating_sub(16),
                            Some(point.col.saturating_add(16)),
                        ),
                        120,
                    )),
                );

            if let Some((grapheme, start_col, end_col)) = visual_grapheme_at(&line, point.col) {
                event = event
                    .with_data(
                        format!("{key_prefix}grapheme"),
                        Value::from(truncate_for_log(&grapheme, 16)),
                    )
                    .with_data(
                        format!("{key_prefix}grapheme_start_col"),
                        Value::from(u64::try_from(start_col).unwrap_or(u64::MAX)),
                    )
                    .with_data(
                        format!("{key_prefix}grapheme_end_col"),
                        Value::from(u64::try_from(end_col).unwrap_or(u64::MAX)),
                    );
            }
        }

        if let Some(line) = render_line {
            event = event.with_data(
                format!("{key_prefix}line_render_preview"),
                Value::from(truncate_for_log(&line, 120)),
            );
        }

        event
    }

    fn log_preview_drag_started(&self, x: u16, y: u16, point: Option<TextSelectionPoint>) {
        let mut event = LogEvent::new("selection", "preview_drag_started")
            .with_data("x", Value::from(x))
            .with_data("y", Value::from(y))
            .with_data("mapped", Value::from(point.is_some()))
            .with_data("interactive", Value::from(self.interactive.is_some()))
            .with_data("mode", Value::from(Self::mode_name(self.state.mode)))
            .with_data("focus", Value::from(Self::focus_name(self.state.focus)))
            .with_data(
                "preview_offset",
                Value::from(u64::try_from(self.preview.offset).unwrap_or(u64::MAX)),
            );

        if let Some(viewport) = self.preview_content_viewport() {
            event = event
                .with_data("output_x", Value::from(viewport.output_x))
                .with_data("output_y", Value::from(viewport.output_y))
                .with_data(
                    "visible_start",
                    Value::from(u64::try_from(viewport.visible_start).unwrap_or(u64::MAX)),
                )
                .with_data(
                    "visible_end",
                    Value::from(u64::try_from(viewport.visible_end).unwrap_or(u64::MAX)),
                );
        }

        if let Some(point) = point {
            event = event
                .with_data(
                    "line",
                    Value::from(u64::try_from(point.line).unwrap_or(u64::MAX)),
                )
                .with_data(
                    "col",
                    Value::from(u64::try_from(point.col).unwrap_or(u64::MAX)),
                );
            event = self.add_selection_point_snapshot_fields(event, "", point);
            if let Some(line) = self.preview_plain_line(point.line) {
                event = event.with_data("line_preview", Value::from(truncate_for_log(&line, 120)));
            }
            if let Some(render_line) = self.preview.render_lines.get(point.line) {
                event = event.with_data(
                    "render_line_preview",
                    Value::from(truncate_for_log(render_line, 120)),
                );
            }
        }

        self.event_log.log(event);
    }

    fn log_preview_drag_finished(&self, x: u16, y: u16, point: Option<TextSelectionPoint>) {
        let mut event = LogEvent::new("selection", "preview_drag_finished")
            .with_data("x", Value::from(x))
            .with_data("y", Value::from(y))
            .with_data("mapped", Value::from(point.is_some()))
            .with_data(
                "has_selection",
                Value::from(self.preview_selection.has_selection()),
            )
            .with_data("interactive", Value::from(self.interactive.is_some()));

        if let Some(point) = point {
            event = event
                .with_data(
                    "release_line",
                    Value::from(u64::try_from(point.line).unwrap_or(u64::MAX)),
                )
                .with_data(
                    "release_col",
                    Value::from(u64::try_from(point.col).unwrap_or(u64::MAX)),
                );
            event = self.add_selection_point_snapshot_fields(event, "release_", point);
        }

        if let Some(anchor) = self.preview_selection.anchor {
            event = event
                .with_data(
                    "anchor_line",
                    Value::from(u64::try_from(anchor.line).unwrap_or(u64::MAX)),
                )
                .with_data(
                    "anchor_col",
                    Value::from(u64::try_from(anchor.col).unwrap_or(u64::MAX)),
                );
            event = self.add_selection_point_snapshot_fields(event, "anchor_", anchor);
        }

        if let Some(start) = self.preview_selection.start {
            event = event
                .with_data(
                    "start_line",
                    Value::from(u64::try_from(start.line).unwrap_or(u64::MAX)),
                )
                .with_data(
                    "start_col",
                    Value::from(u64::try_from(start.col).unwrap_or(u64::MAX)),
                );
            event = self.add_selection_point_snapshot_fields(event, "start_", start);
        }
        if let Some(end) = self.preview_selection.end {
            event = event
                .with_data(
                    "end_line",
                    Value::from(u64::try_from(end.line).unwrap_or(u64::MAX)),
                )
                .with_data(
                    "end_col",
                    Value::from(u64::try_from(end.col).unwrap_or(u64::MAX)),
                );
            event = self.add_selection_point_snapshot_fields(event, "end_", end);
        }

        if let Some(lines) = self.selected_preview_text_lines() {
            let text = lines.join("\n");
            event = event
                .with_data(
                    "selected_line_count",
                    Value::from(u64::try_from(lines.len()).unwrap_or(u64::MAX)),
                )
                .with_data(
                    "selected_char_count",
                    Value::from(u64::try_from(text.chars().count()).unwrap_or(u64::MAX)),
                )
                .with_data(
                    "selected_preview",
                    Value::from(truncate_for_log(&text, 240)),
                );
        }

        self.event_log.log(event);
    }

    pub(super) fn prepare_preview_selection_drag(&mut self, x: u16, y: u16) {
        let point = self.preview_text_point_at(x, y);
        self.log_preview_drag_started(x, y, point);
        if let Some(point) = point {
            self.preview_selection.prepare_drag(point);
            return;
        }

        self.clear_preview_selection();
    }

    pub(super) fn update_preview_selection_drag(&mut self, x: u16, y: u16) {
        if self.preview_selection.anchor.is_none() {
            return;
        }
        let Some(point) = self.preview_text_point_at(x, y) else {
            return;
        };
        self.preview_selection.handle_drag(point);
    }

    pub(super) fn finish_preview_selection_drag(&mut self, x: u16, y: u16) {
        if self.preview_selection.anchor.is_none() {
            return;
        }
        let release_point = self.preview_text_point_at(x, y);
        if !self.preview_selection.has_selection()
            && let Some(point) = release_point
        {
            self.preview_selection.handle_drag(point);
        }
        self.log_preview_drag_finished(x, y, release_point);
        self.preview_selection.finish_drag();
    }

    pub(super) fn apply_preview_selection_highlight_cells(
        &self,
        frame: &mut Frame,
        inner: Rect,
        visible_plain_lines: &[String],
        visible_start: usize,
    ) {
        if !self.preview_selection.has_selection() {
            return;
        }

        let selection_bg = ui_theme().surface1;
        let output_y = inner.y.saturating_add(PREVIEW_METADATA_ROWS);
        for (offset, line) in visible_plain_lines.iter().enumerate() {
            let line_idx = visible_start.saturating_add(offset);
            let Some((start_col, end_col)) = self.preview_selection.line_selection_cols(line_idx)
            else {
                continue;
            };

            let line_width = line_visual_width(line);
            if line_width == 0 {
                continue;
            }

            let start = start_col.min(line_width.saturating_sub(1));
            let end = end_col
                .unwrap_or_else(|| line_width.saturating_sub(1))
                .min(line_width.saturating_sub(1));
            if end < start {
                continue;
            }

            let y = output_y.saturating_add(u16::try_from(offset).unwrap_or(u16::MAX));
            if y >= inner.bottom() {
                break;
            }

            let x_start = inner
                .x
                .saturating_add(u16::try_from(start).unwrap_or(u16::MAX));
            let x_end = inner
                .x
                .saturating_add(u16::try_from(end).unwrap_or(u16::MAX))
                .min(inner.right().saturating_sub(1));
            if x_start > x_end {
                continue;
            }

            for x in x_start..=x_end {
                if let Some(cell) = frame.buffer.get_mut(x, y) {
                    cell.bg = selection_bg;
                }
            }
        }
    }

    pub(super) fn selected_preview_text_lines(&self) -> Option<Vec<String>> {
        let (start, end) = self.preview_selection.bounds()?;
        let source_len = self
            .preview
            .lines
            .len()
            .max(self.preview.render_lines.len());
        if source_len == 0 {
            return None;
        }

        let start_line = start.line.min(source_len.saturating_sub(1));
        let end_line = end.line.min(source_len.saturating_sub(1));
        if end_line < start_line {
            return None;
        }

        let mut lines = self.preview_plain_lines_range(start_line, end_line.saturating_add(1));
        if lines.is_empty() {
            return None;
        }

        if lines.len() == 1 {
            lines[0] = visual_substring(&lines[0], start.col, Some(end.col));
            return Some(lines);
        }

        lines[0] = visual_substring(&lines[0], start.col, None);
        let last_idx = lines.len().saturating_sub(1);
        lines[last_idx] = visual_substring(&lines[last_idx], 0, Some(end.col));

        Some(lines)
    }

    fn visible_preview_output_lines(&self) -> Vec<String> {
        let Some((_, output_height)) = self.preview_output_dimensions() else {
            return Vec::new();
        };
        let (visible_start, visible_end) =
            self.preview_visible_range_for_height(usize::from(output_height));
        self.preview_plain_lines_range(visible_start, visible_end)
    }

    pub(super) fn copy_interactive_selection_or_visible(&mut self) {
        let selected_lines = self.selected_preview_text_lines();
        let copied_from_selection = selected_lines.is_some();
        let mut lines = selected_lines.unwrap_or_else(|| self.visible_preview_output_lines());
        if lines.is_empty() {
            self.last_tmux_error = Some("no output to copy".to_string());
            self.show_toast("No output to copy", true);
            return;
        }

        while lines.last().is_some_and(|line| line.is_empty()) {
            lines.pop();
        }
        if lines.is_empty() {
            self.last_tmux_error = Some("no output to copy".to_string());
            self.show_toast("No output to copy", true);
            return;
        }
        let text = lines.join("\n");
        self.event_log.log(
            LogEvent::new("selection", "interactive_copy_payload")
                .with_data("from_selection", Value::from(copied_from_selection))
                .with_data(
                    "line_count",
                    Value::from(u64::try_from(lines.len()).unwrap_or(u64::MAX)),
                )
                .with_data(
                    "char_count",
                    Value::from(u64::try_from(text.chars().count()).unwrap_or(u64::MAX)),
                )
                .with_data("preview", Value::from(truncate_for_log(&text, 240))),
        );
        self.copied_text = Some(text.clone());
        match self.clipboard.write_text(&text) {
            Ok(()) => {
                self.last_tmux_error = None;
                self.show_toast(format!("Copied {} line(s)", lines.len()), false);
            }
            Err(error) => {
                self.last_tmux_error = Some(format!("clipboard write failed: {error}"));
                self.show_toast(format!("Copy failed: {error}"), true);
            }
        }
        self.clear_preview_selection();
    }
}
