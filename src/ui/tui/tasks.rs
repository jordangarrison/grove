use std::collections::HashSet;
use std::path::Path;

use crate::application::task_discovery::bootstrap_task_data_for_root_with_sessions;
use crate::infrastructure::adapters::{CommandMultiplexerAdapter, MultiplexerAdapter};

use super::*;

impl GroveApp {
    pub(super) fn load_tasks_from_root(tasks_root: Option<&Path>) -> Vec<Task> {
        let Some(tasks_root) = tasks_root else {
            return Vec::new();
        };

        let running_sessions = CommandMultiplexerAdapter.running_sessions();
        bootstrap_task_data_for_root_with_sessions(tasks_root, &running_sessions).tasks
    }

    pub(super) fn reconcile_task_order(&mut self) {
        let valid_slugs = self
            .tasks
            .iter()
            .map(|task| task.slug.clone())
            .collect::<HashSet<String>>();
        let mut ordered = self
            .task_order
            .iter()
            .filter(|slug| valid_slugs.contains(slug.as_str()))
            .fold(Vec::<String>::new(), |mut acc, slug| {
                if !acc.iter().any(|entry| entry == slug) {
                    acc.push(slug.clone());
                }
                acc
            });
        for task in &self.tasks {
            if !ordered.iter().any(|slug| slug == &task.slug) {
                ordered.push(task.slug.clone());
            }
        }
        self.task_order = ordered;
    }

    pub(super) fn refresh_task_catalog(&mut self) {
        self.tasks = Self::load_tasks_from_root(self.tasks_root.as_deref());
        self.reconcile_task_order();
        self.reorder_workspaces_for_task_order();
    }

    fn task_order_index(&self, task_slug: &str) -> usize {
        self.task_order
            .iter()
            .position(|slug| slug == task_slug)
            .unwrap_or(usize::MAX)
    }

    pub(super) fn ordered_tasks(&self) -> Vec<&Task> {
        let mut tasks = self.tasks.iter().collect::<Vec<&Task>>();
        tasks.sort_by(|left, right| {
            self.task_order_index(left.slug.as_str())
                .cmp(&self.task_order_index(right.slug.as_str()))
                .then_with(|| left.slug.cmp(&right.slug))
        });
        tasks
    }

    pub(super) fn task_for_workspace_path(&self, workspace_path: &Path) -> Option<&Task> {
        self.tasks.iter().find(|task| {
            task.worktrees
                .iter()
                .any(|worktree| refer_to_same_location(worktree.path.as_path(), workspace_path))
        })
    }

    pub(super) fn reorder_workspaces_for_task_order(&mut self) {
        if self.state.workspaces.is_empty() {
            return;
        }

        let selected_workspace_path = self.selected_workspace_path();
        let tasks = self.tasks.clone();
        let task_order = self.task_order.clone();
        self.state.workspaces.sort_by_key(|workspace| {
            task_sort_key(tasks.as_slice(), task_order.as_slice(), workspace)
        });

        if let Some(selected_workspace_path) = selected_workspace_path
            && let Some(index) = self.state.workspaces.iter().position(|workspace| {
                refer_to_same_location(workspace.path.as_path(), selected_workspace_path.as_path())
            })
        {
            self.state.selected_index = index;
        }
    }

    pub(super) fn task_workspace_indices(&self, task: &Task) -> Vec<usize> {
        task.worktrees
            .iter()
            .filter_map(|worktree| {
                self.state.workspaces.iter().position(|workspace| {
                    refer_to_same_location(workspace.path.as_path(), worktree.path.as_path())
                })
            })
            .collect()
    }

    fn selected_task_slug(&self) -> Option<String> {
        let workspace = self.state.selected_workspace()?;
        self.task_for_workspace_path(workspace.path.as_path())
            .map(|task| task.slug.clone())
    }

    pub(super) fn task_reorder_active(&self) -> bool {
        self.task_reorder.is_some()
    }

    pub(super) fn open_task_reorder_mode(&mut self) {
        if self.modal_open() || self.task_reorder_active() {
            return;
        }
        let Some(task_slug) = self.selected_task_slug() else {
            self.show_info_toast("selected workspace is not part of a task");
            return;
        };
        self.reconcile_task_order();
        self.task_reorder = Some(TaskReorderState {
            original_task_order: self.task_order.clone(),
            moving_task_slug: task_slug,
        });
        self.show_info_toast("task reorder mode, j/k or Up/Down move, Enter save, Esc cancel");
    }

    pub(super) fn move_selected_task_in_reorder(&mut self, direction: i8) {
        let Some(reorder) = self.task_reorder.as_ref() else {
            return;
        };
        let Some(current_index) = self
            .task_order
            .iter()
            .position(|slug| slug == &reorder.moving_task_slug)
        else {
            return;
        };
        let next_index = if direction.is_negative() {
            current_index.saturating_sub(1)
        } else {
            current_index
                .saturating_add(1)
                .min(self.task_order.len().saturating_sub(1))
        };
        if next_index == current_index {
            return;
        }

        self.task_order.swap(current_index, next_index);
        self.reorder_workspaces_for_task_order();
    }

    pub(super) fn save_task_reorder(&mut self) {
        if !self.task_reorder_active() {
            return;
        }
        self.reconcile_task_order();
        if let Err(error) = self.save_projects_config() {
            self.show_error_toast(format!("task order save failed: {error}"));
            return;
        }
        self.task_reorder = None;
        self.show_success_toast("task order saved");
    }

    pub(super) fn cancel_task_reorder(&mut self) {
        let Some(reorder) = self.task_reorder.take() else {
            return;
        };
        self.task_order = reorder.original_task_order;
        self.reconcile_task_order();
        self.reorder_workspaces_for_task_order();
        self.show_info_toast("task reorder cancelled");
    }

    pub(super) fn task_header_marker(&self, task: &Task) -> &'static str {
        if self
            .task_reorder
            .as_ref()
            .is_some_and(|reorder| reorder.moving_task_slug == task.slug)
        {
            "↕"
        } else {
            "▾"
        }
    }

    #[cfg(test)]
    pub(super) fn set_tasks_for_test(&mut self, tasks: Vec<Task>) {
        self.tasks = tasks;
        self.reconcile_task_order();
        self.reorder_workspaces_for_task_order();
    }

    #[cfg(test)]
    pub(super) fn set_task_order_for_test(&mut self, task_order: Vec<String>) {
        self.task_order = task_order;
        self.reconcile_task_order();
        self.reorder_workspaces_for_task_order();
    }
}

fn task_sort_key(
    tasks: &[Task],
    task_order: &[String],
    workspace: &Workspace,
) -> (usize, usize, String) {
    if let Some(task) = tasks.iter().find(|task| {
        task.worktrees.iter().any(|worktree| {
            refer_to_same_location(worktree.path.as_path(), workspace.path.as_path())
        })
    }) {
        let worktree_index = task
            .worktrees
            .iter()
            .position(|worktree| {
                refer_to_same_location(worktree.path.as_path(), workspace.path.as_path())
            })
            .unwrap_or(usize::MAX);
        let task_index = task_order
            .iter()
            .position(|slug| slug == &task.slug)
            .unwrap_or(usize::MAX);
        return (
            task_index,
            worktree_index,
            workspace.path.display().to_string(),
        );
    }

    (usize::MAX, usize::MAX, workspace.path.display().to_string())
}
