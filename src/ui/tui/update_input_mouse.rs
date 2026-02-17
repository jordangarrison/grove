use super::*;

impl GroveApp {
    const PREVIEW_MOUSE_SCROLL_LINES: i32 = 3;

    fn sidebar_workspace_index_at_y(&self, y: u16) -> Option<usize> {
        if self.projects.is_empty() {
            return None;
        }

        if matches!(self.discovery_state, DiscoveryState::Error(_))
            && self.state.workspaces.is_empty()
        {
            return None;
        }

        let layout = self.view_layout();
        let sidebar_inner = Block::new().borders(Borders::ALL).inner(layout.sidebar);
        if y < sidebar_inner.y || y >= sidebar_inner.bottom() {
            return None;
        }

        let target_row = usize::from(y.saturating_sub(sidebar_inner.y));
        let mut visual_row = 0usize;
        for (project_index, project) in self.projects.iter().enumerate() {
            if project_index > 0 {
                if visual_row == target_row {
                    return None;
                }
                visual_row = visual_row.saturating_add(1);
            }

            if visual_row == target_row {
                return None;
            }
            visual_row = visual_row.saturating_add(1);

            let workspace_indices: Vec<usize> = self
                .state
                .workspaces
                .iter()
                .enumerate()
                .filter(|(_, workspace)| {
                    workspace
                        .project_path
                        .as_ref()
                        .is_some_and(|path| project_paths_equal(path, &project.path))
                })
                .map(|(index, _)| index)
                .collect();
            if workspace_indices.is_empty() {
                if visual_row == target_row {
                    return None;
                }
                visual_row = visual_row.saturating_add(1);
                continue;
            }

            for workspace_index in workspace_indices {
                if visual_row == target_row {
                    return Some(workspace_index);
                }
                visual_row = visual_row.saturating_add(usize::from(WORKSPACE_ITEM_HEIGHT));
            }
        }

        None
    }

    fn select_workspace_by_mouse(&mut self, y: u16) {
        let Some(row) = self.sidebar_workspace_index_at_y(y) else {
            return;
        };

        if row != self.state.selected_index {
            self.state.selected_index = row;
            self.handle_workspace_selection_changed();
        }
    }

    pub(super) fn select_workspace_by_index(&mut self, index: usize) {
        if index >= self.state.workspaces.len() {
            return;
        }
        if index == self.state.selected_index {
            return;
        }

        self.state.selected_index = index;
        self.handle_workspace_selection_changed();
    }

    pub(super) fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        if let Some(state) = self.interactive.as_mut() {
            state.note_mouse_event(Instant::now());
        }

        let (region, row_data) = self.hit_region_for_point(mouse_event.x, mouse_event.y);
        let mut event = LogEvent::new("mouse", "event")
            .with_data("x", Value::from(mouse_event.x))
            .with_data("y", Value::from(mouse_event.y))
            .with_data("kind", Value::from(format!("{:?}", mouse_event.kind)))
            .with_data("region", Value::from(Self::hit_region_name(region)))
            .with_data("modal_open", Value::from(self.modal_open()))
            .with_data("interactive", Value::from(self.interactive.is_some()))
            .with_data("divider_drag_active", Value::from(self.divider_drag_active))
            .with_data("focus", Value::from(Self::focus_name(self.state.focus)))
            .with_data("mode", Value::from(Self::mode_name(self.state.mode)));
        if let Some(row_data) = row_data {
            event = event.with_data("row_data", Value::from(row_data));
        }
        if matches!(region, HitRegion::Preview)
            && let Some(point) = self.preview_text_point_at(mouse_event.x, mouse_event.y)
        {
            event = event
                .with_data(
                    "mapped_line",
                    Value::from(u64::try_from(point.line).unwrap_or(u64::MAX)),
                )
                .with_data(
                    "mapped_col",
                    Value::from(u64::try_from(point.col).unwrap_or(u64::MAX)),
                );
            event = self.add_selection_point_snapshot_fields(event, "mapped_", point);
        }
        self.event_log.log(event);

        if self.modal_open() {
            return;
        }

        match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => match region {
                HitRegion::Divider => {
                    self.divider_drag_active = true;
                }
                HitRegion::WorkspaceList => {
                    self.state.focus = PaneFocus::WorkspaceList;
                    self.state.mode = UiMode::List;
                    if let Some(row_data) = row_data {
                        if let Ok(index) = usize::try_from(row_data) {
                            self.select_workspace_by_index(index);
                        }
                    } else {
                        self.select_workspace_by_mouse(mouse_event.y);
                    }
                }
                HitRegion::Preview => {
                    self.state.focus = PaneFocus::Preview;
                    self.state.mode = UiMode::Preview;
                    if self.interactive.is_some() {
                        self.prepare_preview_selection_drag(mouse_event.x, mouse_event.y);
                    } else {
                        self.clear_preview_selection();
                    }
                }
                HitRegion::StatusLine | HitRegion::Header | HitRegion::Outside => {}
            },
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.divider_drag_active {
                    let ratio =
                        clamp_sidebar_ratio(ratio_from_drag(self.viewport_width, mouse_event.x));
                    if ratio != self.sidebar_width_pct {
                        self.sidebar_width_pct = ratio;
                        self.persist_sidebar_ratio();
                        self.sync_interactive_session_geometry();
                    }
                } else if self.interactive.is_some() {
                    self.update_preview_selection_drag(mouse_event.x, mouse_event.y);
                }
            }
            MouseEventKind::Moved => {
                if self.interactive.is_some() && !self.divider_drag_active {
                    self.update_preview_selection_drag(mouse_event.x, mouse_event.y);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.divider_drag_active = false;
                self.finish_preview_selection_drag(mouse_event.x, mouse_event.y);
            }
            MouseEventKind::ScrollUp => {
                if matches!(region, HitRegion::Preview) {
                    self.state.mode = UiMode::Preview;
                    self.state.focus = PaneFocus::Preview;
                    if self.preview_scroll_tab_is_focused() {
                        self.scroll_preview(-Self::PREVIEW_MOUSE_SCROLL_LINES);
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                if matches!(region, HitRegion::Preview) {
                    self.state.mode = UiMode::Preview;
                    self.state.focus = PaneFocus::Preview;
                    if self.preview_scroll_tab_is_focused() {
                        self.scroll_preview(Self::PREVIEW_MOUSE_SCROLL_LINES);
                    }
                }
            }
            _ => {}
        }
    }

    fn persist_sidebar_ratio(&mut self) {
        if let Err(error) = fs::write(
            &self.sidebar_ratio_path,
            serialize_sidebar_ratio(self.sidebar_width_pct),
        ) {
            self.last_tmux_error = Some(format!("sidebar ratio persist failed: {error}"));
        }
    }
}
