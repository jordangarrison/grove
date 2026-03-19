use super::*;
use std::time::Instant;

impl GroveApp {
    pub(super) fn open_performance_dialog(&mut self) {
        if self.modal_open() {
            return;
        }

        self.refresh_process_metrics(Instant::now());
        self.set_performance_dialog(PerformanceDialogState);
    }
}
