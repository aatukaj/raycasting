use crate::math::*;
pub struct Surface {
    pub width: usize,
    pub height: usize,
    pub pixel_buffer: Vec<u32>,
}

impl Surface {
    pub fn empty(width: usize, height: usize) -> Self {
        Surface {
            width,
            height,
            pixel_buffer: vec![0; width * height],
        }
    }

    pub fn fill(&mut self, value: u32) {
        self.pixel_buffer.fill(value);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, value: u32) -> Result<(), ()> {
        let index = x as usize + y as usize * self.width;
        if index < self.pixel_buffer.len() {
            self.pixel_buffer[index] = value;
            return Ok(());
        }
        return Err(());
    }

    pub fn blit(&mut self, source: &Surface, x: i32, y: i32) {
        for (i, &val) in source.pixel_buffer.iter().enumerate() {
            let x = i as i32 % source.width as i32 + x;
            let y = i as i32 / source.width as i32 + y;
            if x < 0 || y < 0 {
                continue;
            }
            let index = x as usize + y as usize * self.width;
            if index < self.pixel_buffer.len() && val >> 24 == 0xff {
                self.pixel_buffer[index] = val;
            }
        }
    }
    pub fn blit_scaled(&mut self, source: &Surface, pos: Vec2<i32>, scale: f32) {
        let step = 1.0 / scale;
        let mut x = 0.0;

        loop {
            let mut y = 0.0;
            loop {
                let ix = (x / step) as i32 + pos.x - (source.width as f32 * (0.5 * scale)) as i32;
                let iy = (y / step) as i32 + pos.y - (source.height as f32 * (0.5 * scale)) as i32;

                if ix >= 0 && ix < self.width as i32 && iy >= 0 && iy < self.height as i32 {
                    let val = source.pixel_buffer[x as usize + y as usize * source.width];
                    if val != 0 {
                        self.pixel_buffer[ix as usize + iy as usize * self.width] = val
                    }
                }
                y += step;
                if y >= source.height as f32 {
                    break;
                }
            }
            x += step;
            if x >= source.width as f32 {
                break;
            }
        }
    }
}
