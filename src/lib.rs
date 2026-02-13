pub mod adapters;
pub mod agent_runtime;
pub mod domain;
pub mod hardening;
pub mod interactive;
pub mod mouse;
pub mod preview;
pub mod state;
pub mod tui;
pub mod workspace_lifecycle;

pub fn hello_message(app_name: &str) -> String {
    format!("Hello from {app_name}.")
}

pub fn run_tui() -> std::io::Result<()> {
    tui::run()
}

#[cfg(test)]
mod tests {
    use super::hello_message;

    #[test]
    fn hello_message_includes_app_name() {
        assert_eq!(hello_message("grove"), "Hello from grove.");
    }
}
