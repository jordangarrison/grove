pub fn hello_message(app_name: &str) -> String {
    format!("Hello from {app_name}.")
}

#[cfg(test)]
mod tests {
    use super::hello_message;

    #[test]
    fn hello_message_includes_app_name() {
        assert_eq!(hello_message("grove"), "Hello from grove.");
    }
}
