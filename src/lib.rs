pub mod adapters;
pub mod domain;
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
