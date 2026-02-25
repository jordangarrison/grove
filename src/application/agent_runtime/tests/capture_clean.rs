use super::*;

#[test]
fn capture_change_detects_mouse_fragment_noise() {
    let first = evaluate_capture_change(None, "hello\u{1b}[?1000h\u{1b}[<35;192;47M");
    assert!(first.changed_raw);
    assert!(first.changed_cleaned);

    let second = evaluate_capture_change(Some(&first.digest), "hello\u{1b}[?1000l");
    assert!(second.changed_raw);
    assert!(!second.changed_cleaned);
    assert_eq!(second.cleaned_output, "hello");

    let third = evaluate_capture_change(Some(&second.digest), "hello world");
    assert!(third.changed_cleaned);
}

#[test]
fn capture_change_first_capture_marks_changed() {
    let change: CaptureChange = evaluate_capture_change(None, "one");
    assert!(change.changed_raw);
    assert!(change.changed_cleaned);
}

#[test]
fn capture_change_strips_ansi_control_sequences() {
    let raw = "A\u{1b}[31mB\u{1b}[39m C\u{1b}]0;title\u{7}\n";
    let change = evaluate_capture_change(None, raw);
    assert_eq!(change.cleaned_output, "AB C\n");
}

#[test]
fn capture_change_strips_terminal_control_bytes() {
    let raw = "A\u{000e}B\u{000f}C\r\n";
    let change = evaluate_capture_change(None, raw);
    assert_eq!(change.cleaned_output, "ABC\n");
    assert_eq!(change.render_output, "ABC\n");
}

#[test]
fn capture_change_ignores_truncated_partial_mouse_fragments() {
    let first = evaluate_capture_change(None, "prompt [<65;103;31");
    assert_eq!(first.cleaned_output, "prompt ");

    let second = evaluate_capture_change(Some(&first.digest), "prompt [<65;103;32");
    assert!(!second.changed_cleaned);
    assert_eq!(second.cleaned_output, "prompt ");
}

#[test]
fn strip_mouse_fragments_removes_terminal_modes_and_preserves_normal_brackets() {
    assert_eq!(strip_mouse_fragments("value[?1002h"), "value");
    assert_eq!(strip_mouse_fragments("keep [test]"), "keep [test]");
}

#[test]
fn strip_mouse_fragments_removes_boundary_prefixed_partial_sequences() {
    assert_eq!(strip_mouse_fragments("prompt M[<64;107;16M"), "prompt ");
    assert_eq!(strip_mouse_fragments("prompt m[<65;107;14"), "prompt ");
}
