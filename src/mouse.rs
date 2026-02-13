#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutMetrics {
    pub total_width: u16,
    pub total_height: u16,
    pub sidebar_width_pct: u16,
    pub status_line_height: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitRegion {
    WorkspaceList,
    Preview,
    Divider,
    StatusLine,
    Outside,
}

pub fn clamp_sidebar_ratio(ratio_pct: u16) -> u16 {
    ratio_pct.clamp(20, 60)
}

pub fn sidebar_width(total_width: u16, ratio_pct: u16) -> u16 {
    let clamped = clamp_sidebar_ratio(ratio_pct);
    ((total_width as u32 * clamped as u32) / 100) as u16
}

pub fn ratio_from_drag(total_width: u16, drag_x: u16) -> u16 {
    if total_width == 0 {
        return 20;
    }

    let ratio = ((drag_x as u32 * 100) / total_width as u32) as u16;
    clamp_sidebar_ratio(ratio)
}

pub fn divider_x(total_width: u16, ratio_pct: u16) -> u16 {
    sidebar_width(total_width, ratio_pct)
}

pub fn hit_test(layout: LayoutMetrics, x: u16, y: u16) -> HitRegion {
    if x >= layout.total_width || y >= layout.total_height {
        return HitRegion::Outside;
    }

    let status_y = layout
        .total_height
        .saturating_sub(layout.status_line_height);
    if y >= status_y {
        return HitRegion::StatusLine;
    }

    let divider = divider_x(layout.total_width, layout.sidebar_width_pct);
    if x == divider {
        return HitRegion::Divider;
    }
    if x < divider {
        return HitRegion::WorkspaceList;
    }

    HitRegion::Preview
}

pub fn modal_blocks_background_input(modal_open: bool) -> bool {
    modal_open
}

pub fn serialize_sidebar_ratio(ratio_pct: u16) -> String {
    clamp_sidebar_ratio(ratio_pct).to_string()
}

pub fn parse_sidebar_ratio(value: &str) -> Option<u16> {
    let parsed = value.trim().parse::<u16>().ok()?;
    Some(clamp_sidebar_ratio(parsed))
}

#[cfg(test)]
mod tests {
    use super::{
        HitRegion, LayoutMetrics, clamp_sidebar_ratio, hit_test, modal_blocks_background_input,
        parse_sidebar_ratio, ratio_from_drag, serialize_sidebar_ratio,
    };

    #[test]
    fn hit_test_maps_list_preview_divider_and_status_regions() {
        let layout = LayoutMetrics {
            total_width: 100,
            total_height: 40,
            sidebar_width_pct: 30,
            status_line_height: 1,
        };

        assert_eq!(hit_test(layout, 10, 5), HitRegion::WorkspaceList);
        assert_eq!(hit_test(layout, 30, 5), HitRegion::Divider);
        assert_eq!(hit_test(layout, 40, 5), HitRegion::Preview);
        assert_eq!(hit_test(layout, 40, 39), HitRegion::StatusLine);
        assert_eq!(hit_test(layout, 140, 5), HitRegion::Outside);
    }

    #[test]
    fn drag_ratio_is_clamped_between_twenty_and_sixty_percent() {
        assert_eq!(ratio_from_drag(100, 5), 20);
        assert_eq!(ratio_from_drag(100, 50), 50);
        assert_eq!(ratio_from_drag(100, 90), 60);
    }

    #[test]
    fn ratio_serialization_round_trips_with_clamp() {
        assert_eq!(serialize_sidebar_ratio(15), "20");
        assert_eq!(parse_sidebar_ratio("15"), Some(20));
        assert_eq!(parse_sidebar_ratio("55"), Some(55));
        assert_eq!(parse_sidebar_ratio("88"), Some(60));
        assert_eq!(parse_sidebar_ratio("nope"), None);
    }

    #[test]
    fn modal_guard_blocks_background_input_when_open() {
        assert!(modal_blocks_background_input(true));
        assert!(!modal_blocks_background_input(false));
    }

    #[test]
    fn clamp_sidebar_ratio_bounds_values() {
        assert_eq!(clamp_sidebar_ratio(0), 20);
        assert_eq!(clamp_sidebar_ratio(33), 33);
        assert_eq!(clamp_sidebar_ratio(100), 60);
    }
}
