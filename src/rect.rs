use crate::math::Vec2;


#[derive(PartialEq, Debug, Clone)]
pub struct Rect {
    pub pos: Vec2<f32>,
    pub size: f32,
}
impl Rect {
    pub fn get_corners(&self) -> [Vec2<f32>; 4] {
        let h_size = self.size / 2.0;
        [
            self.pos + Vec2::new(h_size, h_size),
            self.pos + Vec2::new(-h_size, h_size),
            self.pos + Vec2::new(-h_size, -h_size),
            self.pos + Vec2::new(h_size, -h_size),
        ]
    }

    pub fn get_top(&self) -> f32 {
        self.pos.y - self.size / 2.0
    }
    pub fn get_bottom(&self) -> f32 {
        self.pos.y + self.size / 2.0
    }
    pub fn get_left(&self) -> f32 {
        self.pos.x - self.size / 2.0
    }
    pub fn get_right(&self) -> f32 {
        self.pos.x + self.size / 2.0
    }
    pub fn set_top(&mut self, y: f32) {
        self.pos.y = y + self.size / 2.0;
    }
    pub fn set_bottom(&mut self, y: f32) {
        self.pos.y = y - self.size / 2.0;
    }
    pub fn set_left(&mut self, x: f32) {
        self.pos.x = x + self.size / 2.0;
    }
    pub fn set_right(&mut self, x: f32) {
        self.pos.x = x - self.size / 2.0;
    }
    pub fn collide(&self, other: &Rect) -> bool {
        let tot_size = (self.size + other.size) / 2.0;
        (self.pos.x - other.pos.x).abs() <  tot_size && (self.pos.y - other.pos.y).abs() < tot_size
    }
}


