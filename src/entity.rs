use glam::*;

use crate::{
    rect::Rect,
    Component, Game
};

pub struct Entity<'a> {

    pub sprite: Option<&'a str>,
    pub look_angle: f32,
    pub vel: Vec2,

    pub rect: Rect,
    pub collidable: bool,
    components: Option<Vec<Box<dyn Component>>>,
    pub alive: bool,
    pub id: u32,
}
impl<'a> Entity<'a> {
    pub fn new(
        pos: Vec2,
        sprite: Option<&'a str>,
        vel: Vec2,
        size: f32,
        collidable: bool,
        components: Vec<Box<dyn Component>>,
    ) -> Self {
        Entity {
            sprite,
            look_angle: 0.0,
            vel,
            rect: Rect { pos, width: size, height: size },
            collidable,
            components: Some(components),
            alive: true,
            id: 0,
        }
    }

    pub fn update(&mut self, dt: f32, game: &mut Game<'a>) {

        let components = self.components.take();
        if let Some(mut components) = components {
            for component in components.iter_mut() {
                component.update(self, game, dt);
            }
            self.components = Some(components);
        }
    }
}
