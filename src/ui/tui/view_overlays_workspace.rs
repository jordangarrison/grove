use super::*;

impl GroveApp {
    pub(super) fn render_launch_dialog_overlay(&self, frame: &mut Frame, area: Rect) {
        let Some(dialog) = self.launch_dialog.as_ref() else {
            return;
        };
        if area.width < 20 || area.height < 11 {
            return;
        }

        let dialog_width = area.width.saturating_sub(8).min(100);
        let dialog_height = 11u16;
        let theme = ui_theme();
        let content_width = usize::from(dialog_width.saturating_sub(2));
        let focused = |field| dialog.focused_field == field;
        let unsafe_focused = focused(LaunchDialogField::Unsafe);
        let unsafe_state = if dialog.skip_permissions {
            "on, bypass approvals and sandbox"
        } else {
            "off, standard safety checks"
        };
        let start_focused = focused(LaunchDialogField::StartButton);
        let cancel_focused = focused(LaunchDialogField::CancelButton);
        let body = FtText::from_lines(vec![
            FtLine::from_spans(vec![FtSpan::styled(
                pad_or_truncate_to_display_width("Launch profile", content_width),
                Style::new().fg(theme.overlay0),
            )]),
            FtLine::raw(""),
            modal_labeled_input_row(
                content_width,
                theme,
                "Prompt",
                dialog.prompt.as_str(),
                "Describe initial task for the agent",
                focused(LaunchDialogField::Prompt),
            ),
            modal_labeled_input_row(
                content_width,
                theme,
                "PreLaunch",
                dialog.pre_launch_command.as_str(),
                "Optional command to run before launch",
                focused(LaunchDialogField::PreLaunchCommand),
            ),
            modal_focus_badged_row(
                content_width,
                theme,
                "Unsafe",
                unsafe_state,
                unsafe_focused,
                theme.peach,
                if dialog.skip_permissions {
                    theme.red
                } else {
                    theme.text
                },
            ),
            FtLine::raw(""),
            modal_actions_row(
                content_width,
                theme,
                "Start",
                "Cancel",
                start_focused,
                cancel_focused,
            ),
            FtLine::from_spans(vec![FtSpan::styled(
                pad_or_truncate_to_display_width(
                    "Tab move, Space toggle unsafe, Enter start, Esc cancel",
                    content_width,
                ),
                Style::new().fg(theme.overlay0),
            )]),
        ]);
        let content = OverlayModalContent {
            title: "Start Agent",
            body,
            theme,
            border_color: theme.mauve,
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
            .hit_id(HitId::new(HIT_ID_LAUNCH_DIALOG))
            .render(area, frame);
    }

    pub(super) fn render_delete_dialog_overlay(&self, frame: &mut Frame, area: Rect) {
        let Some(dialog) = self.delete_dialog.as_ref() else {
            return;
        };
        if area.width < 24 || area.height < 12 {
            return;
        }

        let dialog_width = area.width.saturating_sub(8).min(96);
        let dialog_height = 16u16;
        let theme = ui_theme();
        let content_width = usize::from(dialog_width.saturating_sub(2));
        let focused = |field| dialog.focused_field == field;
        let warning_lines = if dialog.is_missing {
            (
                "  • Directory already removed",
                "  • Clean up git worktree metadata",
            )
        } else {
            (
                "  • Remove the working directory",
                "  • Uncommitted changes will be lost",
            )
        };
        let cleanup_focused = focused(DeleteDialogField::DeleteLocalBranch);
        let cleanup_state = if dialog.delete_local_branch {
            format!("enabled, remove '{}' branch locally", dialog.branch)
        } else {
            "disabled, keep local branch".to_string()
        };
        let delete_focused = focused(DeleteDialogField::DeleteButton);
        let cancel_focused = focused(DeleteDialogField::CancelButton);
        let delete_hint = pad_or_truncate_to_display_width(
            "Tab move, Space toggle branch cleanup, Enter or D delete, Esc cancel",
            content_width,
        );
        let path = dialog.path.display().to_string();
        let body = FtText::from_lines(vec![
            FtLine::from_spans(vec![FtSpan::styled(
                pad_or_truncate_to_display_width("Deletion plan", content_width),
                Style::new().fg(theme.overlay0),
            )]),
            FtLine::raw(""),
            modal_static_badged_row(
                content_width,
                theme,
                "Name",
                dialog.workspace_name.as_str(),
                theme.blue,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "Branch",
                dialog.branch.as_str(),
                theme.blue,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "Path",
                path.as_str(),
                theme.blue,
                theme.overlay0,
            ),
            FtLine::from_spans(vec![FtSpan::styled(
                pad_or_truncate_to_display_width("  [Risk] Changes are destructive", content_width),
                Style::new().fg(theme.peach).bold(),
            )]),
            FtLine::from_spans(vec![FtSpan::styled(
                warning_lines.0,
                Style::new().fg(theme.subtext0),
            )]),
            FtLine::from_spans(vec![FtSpan::styled(
                warning_lines.1,
                Style::new().fg(theme.subtext0),
            )]),
            FtLine::raw(""),
            modal_focus_badged_row(
                content_width,
                theme,
                "BranchCleanup",
                cleanup_state.as_str(),
                cleanup_focused,
                theme.peach,
                if dialog.delete_local_branch {
                    theme.red
                } else {
                    theme.text
                },
            ),
            FtLine::raw(""),
            modal_actions_row(
                content_width,
                theme,
                "Delete",
                "Cancel",
                delete_focused,
                cancel_focused,
            ),
            FtLine::from_spans(vec![FtSpan::styled(
                delete_hint,
                Style::new().fg(theme.overlay0),
            )]),
        ]);

        let content = OverlayModalContent {
            title: "Delete Worktree?",
            body,
            theme,
            border_color: theme.red,
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
            .hit_id(HitId::new(HIT_ID_DELETE_DIALOG))
            .render(area, frame);
    }

    pub(super) fn render_merge_dialog_overlay(&self, frame: &mut Frame, area: Rect) {
        let Some(dialog) = self.merge_dialog.as_ref() else {
            return;
        };
        if area.width < 28 || area.height < 14 {
            return;
        }

        let dialog_width = area.width.saturating_sub(8).min(98);
        let dialog_height = 17u16;
        let theme = ui_theme();
        let content_width = usize::from(dialog_width.saturating_sub(2));
        let focused = |field| dialog.focused_field == field;
        let cleanup_workspace_focused = focused(MergeDialogField::CleanupWorkspace);
        let cleanup_workspace_state = if dialog.cleanup_workspace {
            "enabled, remove workspace directory".to_string()
        } else {
            "disabled, keep workspace directory".to_string()
        };
        let cleanup_branch_focused = focused(MergeDialogField::CleanupLocalBranch);
        let cleanup_branch_state = if dialog.cleanup_local_branch {
            format!("enabled, delete '{}' branch", dialog.workspace_branch)
        } else {
            "disabled, keep local branch".to_string()
        };
        let merge_focused = focused(MergeDialogField::MergeButton);
        let cancel_focused = focused(MergeDialogField::CancelButton);
        let merge_hint = pad_or_truncate_to_display_width(
            "Tab move, Space toggle cleanup, Enter or m merge, Esc cancel",
            content_width,
        );
        let path = dialog.workspace_path.display().to_string();
        let body = FtText::from_lines(vec![
            FtLine::from_spans(vec![FtSpan::styled(
                pad_or_truncate_to_display_width("Merge plan", content_width),
                Style::new().fg(theme.overlay0),
            )]),
            FtLine::raw(""),
            modal_static_badged_row(
                content_width,
                theme,
                "Name",
                dialog.workspace_name.as_str(),
                theme.blue,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "Branch",
                dialog.workspace_branch.as_str(),
                theme.blue,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "Base",
                dialog.base_branch.as_str(),
                theme.blue,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "Path",
                path.as_str(),
                theme.blue,
                theme.overlay0,
            ),
            FtLine::raw(""),
            modal_focus_badged_row(
                content_width,
                theme,
                "CleanupWorktree",
                cleanup_workspace_state.as_str(),
                cleanup_workspace_focused,
                theme.peach,
                if dialog.cleanup_workspace {
                    theme.red
                } else {
                    theme.text
                },
            ),
            modal_focus_badged_row(
                content_width,
                theme,
                "CleanupBranch",
                cleanup_branch_state.as_str(),
                cleanup_branch_focused,
                theme.peach,
                if dialog.cleanup_local_branch {
                    theme.red
                } else {
                    theme.text
                },
            ),
            FtLine::raw(""),
            modal_actions_row(
                content_width,
                theme,
                "Merge",
                "Cancel",
                merge_focused,
                cancel_focused,
            ),
            FtLine::from_spans(vec![FtSpan::styled(
                merge_hint,
                Style::new().fg(theme.overlay0),
            )]),
        ]);

        let content = OverlayModalContent {
            title: "Merge Workspace?",
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
            .hit_id(HitId::new(HIT_ID_MERGE_DIALOG))
            .render(area, frame);
    }

    pub(super) fn render_update_from_base_dialog_overlay(&self, frame: &mut Frame, area: Rect) {
        let Some(dialog) = self.update_from_base_dialog.as_ref() else {
            return;
        };
        if area.width < 28 || area.height < 12 {
            return;
        }

        let dialog_width = area.width.saturating_sub(8).min(96);
        let dialog_height = 14u16;
        let theme = ui_theme();
        let content_width = usize::from(dialog_width.saturating_sub(2));
        let focused = |field| dialog.focused_field == field;
        let update_focused = focused(UpdateFromBaseDialogField::UpdateButton);
        let cancel_focused = focused(UpdateFromBaseDialogField::CancelButton);
        let update_hint = pad_or_truncate_to_display_width(
            "Tab move, h/l switch buttons, Enter or u update, Esc cancel",
            content_width,
        );
        let path = dialog.workspace_path.display().to_string();
        let body = FtText::from_lines(vec![
            FtLine::from_spans(vec![FtSpan::styled(
                pad_or_truncate_to_display_width("Update plan", content_width),
                Style::new().fg(theme.overlay0),
            )]),
            FtLine::raw(""),
            modal_static_badged_row(
                content_width,
                theme,
                "Name",
                dialog.workspace_name.as_str(),
                theme.blue,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "Branch",
                dialog.workspace_branch.as_str(),
                theme.blue,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "Base",
                dialog.base_branch.as_str(),
                theme.blue,
                theme.text,
            ),
            modal_static_badged_row(
                content_width,
                theme,
                "Path",
                path.as_str(),
                theme.blue,
                theme.overlay0,
            ),
            FtLine::from_spans(vec![FtSpan::styled(
                pad_or_truncate_to_display_width(
                    "  Strategy: git merge --no-ff <base> into workspace branch",
                    content_width,
                ),
                Style::new().fg(theme.subtext0),
            )]),
            FtLine::raw(""),
            modal_actions_row(
                content_width,
                theme,
                "Update",
                "Cancel",
                update_focused,
                cancel_focused,
            ),
            FtLine::from_spans(vec![FtSpan::styled(
                update_hint,
                Style::new().fg(theme.overlay0),
            )]),
        ]);

        let content = OverlayModalContent {
            title: "Update From Base?",
            body,
            theme,
            border_color: theme.teal,
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
            .hit_id(HitId::new(HIT_ID_UPDATE_FROM_BASE_DIALOG))
            .render(area, frame);
    }
}
