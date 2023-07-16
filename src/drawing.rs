#![allow(dead_code)]
use crate::Surface;
use glam::*;

struct LineDrawer {
    end_x: i32,

    dx: i32,
    dy: i32,
    x: i32,
    y: i32,
    yi: i32,
    d: i32,
    flipped: bool,
}
impl LineDrawer {
    fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        let mut flipped = false;

        let (x0, y0, x1, y1) = if (y1 - y0).abs() > (x1 - x0).abs() {
            flipped = true;
            (y0, x0, y1, x1)
        } else {
            (x0, y0, x1, y1)
        };

        let (x0, y0, x1, y1) = if x1 < x0 {
            (x1, y1, x0, y0)
        } else {
            (x0, y0, x1, y1)
        };

        let dx = x1 - x0;
        let mut dy = y1 - y0;
        let mut yi = 1;
        if dy < 0 {
            yi = -1;
            dy = -dy;
        }

        LineDrawer {
            end_x: x1,

            dx,
            dy,
            x: x0,
            y: y0,
            yi,
            d: 2 * dy - dx,
            flipped,
        }
    }
}

impl Iterator for LineDrawer {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        self.x += 1;
        if self.x > self.end_x {
            return None;
        }
        if self.d > 0 {
            self.y += self.yi;
            self.d += 2 * (self.dy - self.dx);
        } else {
            self.d += 2 * self.dy;
        }
        if self.flipped {
            Some((self.y, self.x))
        } else {
            Some((self.x, self.y))
        }
    }
}


#[inline]
pub fn val_from_rgb(r: u32, g: u32, b: u32) -> u32 {
    b.min(255) | (g.min(255) << 8) | (r.min(255) << 16) | (255 << 24)
}
pub fn val_from_rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    (b as u32) | ((g as u32) << 8) | ((r as u32) << 16) as u32 | (a as u32) << 24
}
pub fn draw_dotted_line(surf: &mut Surface, p0: IVec2, p1: IVec2, value: u32) {
    for (x, y) in LineDrawer::new(p0.x, p0.y, p1.x, p1.y)
        .enumerate()
        .filter_map(|(i, el)| ((0..3).contains(&(i % 6))).then_some(el))
    {
        let _ = surf.set_pixel(x as u32, y as u32, value);
    }
}

pub fn draw_rect(surf: &mut Surface, pos: IVec2, size: IVec2, value: u32) {
    for x in pos.x.max(0)..(pos.x + size.x).min(surf.width as i32) {
        for y in pos.y.max(0)..(pos.y + size.y).min(surf.height as i32) {
            surf.set_pixel(x as u32, y as u32, value).unwrap();
        }
    }
}

pub fn draw_line(surf: &mut Surface, p0: IVec2, p1: IVec2, value: u32) {
    for (x, y) in LineDrawer::new(p0.x, p0.y, p1.x, p1.y) {
        let _ = surf.set_pixel(x as u32, y as u32, value);
    }
}
