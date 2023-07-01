use crate::{math::Vec2, rect::Rect, surface::Surface, tile_map::TileMap};

pub struct Entity<'a> {
    pub pos: Vec2<f32>,
    pub sprite: Option<&'a str>,
    pub look_angle: f32,
    pub vel: Vec2<f32>,

    rect: Rect,
    collidable: bool,
}
impl<'a> Entity<'a> {
    pub fn new(
        pos: Vec2<f32>,
        sprite: Option<&'a str>,
        vel: Vec2<f32>,
        size: f32,
        collidable: bool,
    ) -> Self {
        Entity {
            pos,
            sprite,
            look_angle: 0.0,
            vel,
            rect: Rect { pos, size },
            collidable,
        }
    }

    pub fn update(&mut self, dt: f32, tile_map: &TileMap, entities: &[Entity]) {
        self.rect.pos = self.pos;
        self.rect.pos.x += self.vel.x * dt;
        let h_size = self.rect.size / 2.0;
        let cols = tile_map.get_collisions(&self.rect);
        for col in cols {
            if self.vel.x > 0.0 {
                self.rect.pos.x = col.x as f32 - h_size - 0.00420; //i dont like the arbitrary subtraction but it fixes a bug
            } else {
                self.rect.pos.x = col.x as f32 + 1.0 + h_size;
            }
        }
        for entity in entities.iter() {
            if entity.collidable && self.rect.collide(&entity.rect) {
                if self.vel.x > 0.0 {
                    self.rect.set_right(entity.rect.get_left());
                } else {
                    self.rect.set_left(entity.rect.get_right());
                }
            }
        }

        self.rect.pos.y += self.vel.y * dt;
        let cols = tile_map.get_collisions(&self.rect);
        for col in cols {
            if self.vel.y > 0.0 {
                self.rect.pos.y = col.y as f32 - h_size - 0.00420;
            } else {
                self.rect.pos.y = col.y as f32 + 1.0 + h_size;
            }
        }
        for entity in entities.iter() {
            if entity.collidable && self.rect.collide(&entity.rect) {
                if self.vel.y > 0.0 {
                    self.rect.set_bottom(entity.rect.get_top());
                } else {
                    self.rect.set_top(entity.rect.get_bottom());
                }
            }
        }

        self.pos = self.rect.pos;
    }
}
