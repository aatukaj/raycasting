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

    pub fn blit(&mut self, source: &Surface, x: usize, y: usize) {
        for (i, &val) in source.pixel_buffer.iter().enumerate() {
            let x = i % source.width + x;
            let y = i / source.width + y;
            let index = x + y * self.width;
            if index < self.pixel_buffer.len() && val >> 24 == 0xff {
                self.pixel_buffer[index] = val;
            }
        }
    }
}