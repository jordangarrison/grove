use super::{clamp_sidebar_ratio, parse_sidebar_ratio, ratio_from_drag, serialize_sidebar_ratio};

#[test]
fn drag_ratio_is_clamped_between_ten_and_sixty_percent() {
    assert_eq!(ratio_from_drag(100, 5), 10);
    assert_eq!(ratio_from_drag(100, 50), 50);
    assert_eq!(ratio_from_drag(100, 90), 60);
}

#[test]
fn ratio_serialization_round_trips_with_clamp() {
    assert_eq!(serialize_sidebar_ratio(5), "10");
    assert_eq!(parse_sidebar_ratio("5"), Some(10));
    assert_eq!(parse_sidebar_ratio("55"), Some(55));
    assert_eq!(parse_sidebar_ratio("88"), Some(60));
    assert_eq!(parse_sidebar_ratio("nope"), None);
}

#[test]
fn clamp_sidebar_ratio_bounds_values() {
    assert_eq!(clamp_sidebar_ratio(0), 10);
    assert_eq!(clamp_sidebar_ratio(33), 33);
    assert_eq!(clamp_sidebar_ratio(100), 60);
}
