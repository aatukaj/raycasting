use crate::{
    math::Vec2, rect::Rect, surface::Surface, tile_map::TileMap, BasicCollisionComponent,
    Component, Game,
};

pub struct Entity<'a> {
    pub pos: Vec2<f32>,
    pub sprite: Option<&'a str>,
    pub look_angle: f32,
    pub vel: Vec2<f32>,

    pub rect: Rect,
    pub collidable: bool,
    components: Vec<Box<dyn Component>>,
    pub alive: bool
}
impl<'a> Entity<'a> {
    pub fn new(
        pos: Vec2<f32>,
        sprite: Option<&'a str>,
        vel: Vec2<f32>,
        size: f32,
        collidable: bool,
        components: Vec<Box<dyn Component>>
    ) -> Self {
        Entity {
            pos,
            sprite,
            look_angle: 0.0,
            vel,
            rect: Rect { pos, size },
            collidable,
            components,
            alive: true
        }
    }

    pub fn update(&mut self, dt: f32, game: &mut Game) {
        for i in (0..=0).chain(0..(self.components.len() - 1)) {
            let component = self.components.swap_remove(i);
            component.update(self, game, dt);
            self.components.push(component);
        }
    }
}
