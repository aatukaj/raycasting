use glam::*;



#[derive(Debug)]
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
    pub fn blit_scaled(&mut self, source: &Surface, pos: IVec2, scale: f32) {
        let scaled_width = (source.width as f32 * scale) as i32;
        let scaled_height = (source.height as f32 * scale) as i32;
        let offset_x = pos.x - scaled_width / 2;
        let offset_y = pos.y - scaled_height / 2;
        for y in offset_y.max(0)..(offset_y + scaled_height).min(self.height as i32) {
            for x in offset_x.max(0)..(offset_x + scaled_width).min(self.width as i32) {
                let index_self = x as usize + y as usize * self.width;
                let index_source = ((x - offset_x) as f32 / scale) as usize
                    + ((y - offset_y) as f32 / scale) as usize * source.width;
                let val = source.pixel_buffer[index_source];
                if val != 0 {
                    self.pixel_buffer[index_self] = val;
                }
            }
        }
    }
}
