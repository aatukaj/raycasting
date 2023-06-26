use crate::Surface;
use crate::Vec2;

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


pub fn val_from_rgb(r: u32, g: u32, b: u32) -> u32 {
    b + (g << 8) + (r << 16) + (255 << 24)
}


pub fn draw_dotted_line(surf: &mut Surface, p0: Vec2<i32>, p1: Vec2<i32>, value: u32) {
    for (x, y) in LineDrawer::new(p0.x, p0.y, p1.x, p1.y)
        .enumerate()
        .filter_map(|(i, el)| ((0..3).contains(&(i % 6))).then(|| el))
    {
        let _ = surf.set_pixel(x as u32, y as u32, value);
    }
}

pub fn draw_rect(surf: &mut Surface, pos : Vec2<i32>, size: Vec2<i32>, value: u32) {
    for x in pos.x.max(0)..(pos.x + size.x).min(surf.width as i32) {
        for y in pos.y.max(0)..(pos.y + size.y).min(surf.height as i32) {
            surf.set_pixel(x as u32, y as u32, value).unwrap();
        }
    }
}

pub fn draw_line(surf: &mut Surface, p0: Vec2<i32>, p1: Vec2<i32>, value: u32) {
    for (x, y) in LineDrawer::new(p0.x, p0.y, p1.x, p1.y) {
        let _ = surf.set_pixel(x as u32, y as u32, value);
    }
}
