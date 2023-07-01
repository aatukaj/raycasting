use crate::math::Vec2;


#[derive(PartialEq, Debug)]
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
        let a_min = Vec2::new(self.get_left(), self.get_top());
        let a_max = Vec2::new(self.get_right(), self.get_bottom());
        let b_min = Vec2::new(other.get_left(), other.get_top());
        let b_max = Vec2::new(other.get_bottom(), other.get_right());

        a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y
    }
}
