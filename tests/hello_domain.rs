#[test]
fn hello_message_uses_expected_format() {
    let message = grove::hello_message("grove");
    assert_eq!(message, "Hello from grove.");
}
