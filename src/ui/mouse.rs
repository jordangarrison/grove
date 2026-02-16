pub fn clamp_sidebar_ratio(ratio_pct: u16) -> u16 {
    ratio_pct.clamp(20, 60)
}

pub fn ratio_from_drag(total_width: u16, drag_x: u16) -> u16 {
    if total_width == 0 {
        return 20;
    }

    let ratio = ((drag_x as u32 * 100) / total_width as u32) as u16;
    clamp_sidebar_ratio(ratio)
}

pub fn serialize_sidebar_ratio(ratio_pct: u16) -> String {
    clamp_sidebar_ratio(ratio_pct).to_string()
}

pub fn parse_sidebar_ratio(value: &str) -> Option<u16> {
    let parsed = value.trim().parse::<u16>().ok()?;
    Some(clamp_sidebar_ratio(parsed))
}

#[cfg(test)]
mod tests;
