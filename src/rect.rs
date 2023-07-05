use glam::*;


#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Rect {
    pub pos: Vec2,
    pub width: f32,
    pub height :f32,
}
impl Rect {
    pub fn get_corners(&self) -> [Vec2; 4] {
        let h_size = self.height / 2.0;
        let w_size = self.width / 2.0;
        [
            self.pos + Vec2::new(w_size, h_size),
            self.pos + Vec2::new(-w_size, h_size),
            self.pos + Vec2::new(-w_size, -h_size),
            self.pos + Vec2::new(w_size, -h_size),
        ]
    }

    pub fn get_top(&self) -> f32 {
        self.pos.y - self.height / 2.0
    }
    pub fn get_bottom(&self) -> f32 {
        self.pos.y + self.height / 2.0
    }
    pub fn get_left(&self) -> f32 {
        self.pos.x - self.width / 2.0
    }
    pub fn get_right(&self) -> f32 {
        self.pos.x + self.width / 2.0
    }
    pub fn set_top(&mut self, y: f32) {
        self.pos.y = y + self.height / 2.0;
    }
    pub fn set_bottom(&mut self, y: f32) {
        self.pos.y = y - self.height / 2.0;
    }
    pub fn set_left(&mut self, x: f32) {
        self.pos.x = x + self.width / 2.0;
    }
    pub fn set_right(&mut self, x: f32) {
        self.pos.x = x - self.width / 2.0;
    }
    pub fn collide(&self, other: &Rect) -> bool {
        let tot_size_x = (self.width + other.width) / 2.0;
        let tot_size_y = (self.height + other.height) / 2.0;
        (self.pos.x - other.pos.x).abs() <  tot_size_x && (self.pos.y - other.pos.y).abs() < tot_size_y
    }
}


