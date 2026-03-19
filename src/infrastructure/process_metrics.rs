use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ProcessMetricsSnapshot {
    pub(crate) cpu_percent: Option<f32>,
    pub(crate) resident_bytes: Option<u64>,
}

impl ProcessMetricsSnapshot {
    pub(crate) const fn unavailable() -> Self {
        Self {
            cpu_percent: None,
            resident_bytes: None,
        }
    }

    pub(crate) fn cpu_display(&self) -> String {
        self.cpu_percent
            .map(|value| format!("{value:.1}%"))
            .unwrap_or_else(|| "unavailable".to_string())
    }

    pub(crate) fn memory_display(&self) -> String {
        self.resident_bytes
            .map(format_memory_bytes)
            .unwrap_or_else(|| "unavailable".to_string())
    }
}

pub(crate) struct ProcessMetricsSampler {
    system: System,
    pid: Pid,
}

impl ProcessMetricsSampler {
    pub(crate) fn new_current() -> Self {
        let pid = Pid::from_u32(std::process::id());
        let refresh = ProcessRefreshKind::nothing().with_cpu().with_memory();
        let system = System::new_with_specifics(RefreshKind::nothing().with_processes(refresh));

        Self { system, pid }
    }

    pub(crate) fn refresh(&mut self) -> ProcessMetricsSnapshot {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[self.pid]),
            true,
            ProcessRefreshKind::nothing().with_cpu().with_memory(),
        );

        self.system
            .process(self.pid)
            .map(|process| ProcessMetricsSnapshot {
                cpu_percent: Some(process.cpu_usage()),
                resident_bytes: Some(process.memory()),
            })
            .unwrap_or_else(ProcessMetricsSnapshot::unavailable)
    }
}

pub(crate) fn format_memory_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;

    if bytes >= GIB {
        return format!("{:.1} GiB", bytes as f64 / GIB as f64);
    }
    if bytes >= MIB {
        return format!("{:.1} MiB", bytes as f64 / MIB as f64);
    }
    if bytes >= KIB {
        return format!("{:.1} KiB", bytes as f64 / KIB as f64);
    }
    format!("{bytes} B")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_memory_bytes_uses_human_readable_units() {
        assert_eq!(format_memory_bytes(0), "0 B");
        assert_eq!(format_memory_bytes(1024), "1.0 KiB");
    }

    #[test]
    fn unavailable_snapshot_formats_as_unavailable() {
        assert_eq!(
            ProcessMetricsSnapshot::unavailable().cpu_display(),
            "unavailable"
        );
        assert_eq!(
            ProcessMetricsSnapshot::unavailable().memory_display(),
            "unavailable"
        );
    }
}
