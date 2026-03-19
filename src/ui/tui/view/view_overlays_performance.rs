use super::view_prelude::*;
use crate::ui::tui::performance::format_duration;
use std::time::Instant;

impl GroveApp {
    pub(super) fn render_performance_dialog_overlay(&self, frame: &mut Frame, area: Rect) {
        if self.performance_dialog().is_none() {
            return;
        }
        if area.width < 56 || area.height < 18 {
            return;
        }

        let dialog_width = area.width.saturating_sub(12).min(88);
        let dialog_height = 18u16;
        let theme = self.active_ui_theme();
        let content_width = usize::from(dialog_width.saturating_sub(2));
        let now = Instant::now();
        let frame_summary = self.frame_timing_summary();
        let process_metrics = self.process_metrics_snapshot();
        let next_tick = self
            .polling
            .next_tick_due_at
            .map(|due_at| due_at.saturating_duration_since(now));
        let next_poll = self
            .polling
            .next_poll_due_at
            .map(|due_at| due_at.saturating_duration_since(now));
        let next_visual = self
            .polling
            .next_visual_due_at
            .map(|due_at| due_at.saturating_duration_since(now));
        let mut lines = vec![
            FtLine::from_spans(vec![FtSpan::styled(
                "Runtime inspection for Grove",
                Style::new().fg(theme.overlay0),
            )]),
            FtLine::raw(""),
            modal_static_badged_row(
                content_width,
                theme,
                "Summary",
                "Frame + Grove process",
                theme.blue,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "CPU",
                process_metrics.cpu_display().as_str(),
                theme.peach,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "Memory",
                process_metrics.memory_display().as_str(),
                theme.peach,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "FPS",
                frame_summary
                    .map(|summary| format!("{:.1} fps", summary.fps_estimate))
                    .unwrap_or_else(|| "warming up".to_string())
                    .as_str(),
                theme.green,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "Frame",
                frame_summary
                    .map(|summary| {
                        format!(
                            "avg {:.1} ms, p95 {:.1} ms",
                            summary.average_ms, summary.p95_ms
                        )
                    })
                    .unwrap_or_else(|| "warming up".to_string())
                    .as_str(),
                theme.green,
                theme.text,
            ),
            FtLine::raw(""),
            modal_static_badged_row(
                content_width,
                theme,
                "Scheduler",
                self.scheduler_reason_summary().as_str(),
                theme.yellow,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "NextTick",
                format_duration(next_tick).as_str(),
                theme.yellow,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "NextPoll",
                format_duration(next_poll).as_str(),
                theme.yellow,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "NextVisual",
                format_duration(next_visual).as_str(),
                theme.yellow,
                theme.text,
            ),
            FtLine::raw(""),
            modal_static_badged_row(
                content_width,
                theme,
                "Sessions",
                "Polling view",
                theme.blue,
                theme.text,
            ),
        ];

        let session_rows = self.session_performance_rows();
        if session_rows.is_empty() {
            lines.push(modal_static_badged_row(
                content_width,
                theme,
                "Row",
                "No active workspace sessions",
                theme.overlay0,
                theme.overlay0,
            ));
        } else {
            for row in session_rows.into_iter().take(3) {
                let value = format!(
                    "{} | {} | {} | {}",
                    row.status, row.cadence, row.role, row.reason
                );
                lines.push(modal_static_badged_row(
                    content_width,
                    theme,
                    row.label.as_str(),
                    value.as_str(),
                    theme.green,
                    theme.text,
                ));
            }
        }
        lines.extend([
            FtLine::raw(""),
            FtLine::from_spans(vec![FtSpan::styled(
                "Close: Esc",
                Style::new().fg(theme.overlay0),
            )]),
        ]);
        let body = FtText::from_lines(lines);

        let content = OverlayModalContent {
            title: "Performance",
            body,
            theme,
            border_color: theme.peach,
        };

        Modal::new(content)
            .size(
                ModalSizeConstraints::new()
                    .min_width(dialog_width)
                    .max_width(dialog_width)
                    .min_height(dialog_height)
                    .max_height(dialog_height),
            )
            .backdrop(BackdropConfig::new(theme.crust, 0.55))
            .hit_id(HitId::new(HIT_ID_PERFORMANCE_DIALOG))
            .render(area, frame);
    }
}
