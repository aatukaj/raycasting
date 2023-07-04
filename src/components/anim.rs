

use super::Component;
use crate::{entity::Entity, Game};

pub struct AnimationComponent {
    pub images: Vec<&'static str>,
    pub time_per_frame: f32,
    pub cur_time: f32,
}

impl Component  for AnimationComponent {
    fn update<'a>(&mut self, entity: &mut Entity<'a>, _game: &mut Game, dt: f32) {
        entity.sprite = Some(self.images[(self.cur_time / self.time_per_frame) as usize]);
        self.cur_time += dt;
        if (self.cur_time / self.time_per_frame) > self.images.len() as f32{
            entity.alive = false;
        }
    }
}
