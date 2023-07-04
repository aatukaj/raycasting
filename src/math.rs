
use crate::drawing::val_from_rgb;

pub fn set_value_brightness(val: u32, brightness: f32) -> u32 {
    let b = val & 0xFF;
    let g = val >> 8 & 0xFF;
    let r = val >> 16 & 0xFF;
    val_from_rgb(
        (r as f32 * brightness) as u32,
        (g as f32 * brightness) as u32,
        (b as f32 * brightness) as u32,
    )
}
