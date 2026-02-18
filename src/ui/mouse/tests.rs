use super::{clamp_sidebar_ratio, ratio_from_drag};

#[test]
fn drag_ratio_is_clamped_between_ten_and_sixty_percent() {
    assert_eq!(ratio_from_drag(100, 5), 10);
    assert_eq!(ratio_from_drag(100, 50), 50);
    assert_eq!(ratio_from_drag(100, 90), 60);
}

#[test]
fn clamp_sidebar_ratio_bounds_values() {
    assert_eq!(clamp_sidebar_ratio(0), 10);
    assert_eq!(clamp_sidebar_ratio(33), 33);
    assert_eq!(clamp_sidebar_ratio(100), 60);
}
