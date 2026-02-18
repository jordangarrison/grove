const SIDEBAR_RATIO_MIN_PCT: u16 = 10;
const SIDEBAR_RATIO_MAX_PCT: u16 = 60;

pub fn clamp_sidebar_ratio(ratio_pct: u16) -> u16 {
    ratio_pct.clamp(SIDEBAR_RATIO_MIN_PCT, SIDEBAR_RATIO_MAX_PCT)
}

pub fn ratio_from_drag(total_width: u16, drag_x: u16) -> u16 {
    if total_width == 0 {
        return SIDEBAR_RATIO_MIN_PCT;
    }

    let ratio = ((drag_x as u32 * 100) / total_width as u32) as u16;
    clamp_sidebar_ratio(ratio)
}

#[cfg(test)]
mod tests;
