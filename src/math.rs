
use crate::drawing::val_from_rgb;



#[inline]
pub fn set_value_brightness(val: u32, brightness: u32) -> u32 {
    let b = val & 0xFF;
    let g = val >> 8 & 0xFF;
    let r = val >> 16 & 0xFF;
    val_from_rgb(
        (r * brightness) / 255,
        (g * brightness) / 255,
        (b * brightness) / 255,
    )
}
