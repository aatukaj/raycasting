use glam::Vec2;

use crate::{entity::Entity, Game};

use super::{AnimationComponent, Component};
pub struct BasicCollisionComponent;
impl Component for BasicCollisionComponent {
    fn update<'a>(&mut self, entity: &mut Entity, game: &mut Game, dt: f32) {
        let old_rect = entity.rect.clone();

        entity.rect.pos.x += entity.vel.x * dt;
        let h_size = entity.rect.size / 2.0;
        let cols = game.tile_map.get_collisions(&entity.rect);
        for col in cols {
            if entity.vel.x > 0.0 {
                entity.rect.pos.x = col.x as f32 - h_size - 0.001; //i dont like the arbitrary subtraction but it fixes a bug
            } else if entity.vel.x < 0.0 {
                entity.rect.pos.x = col.x as f32 + 1.0 + h_size + 0.001;
            }
        }

        for other in game.entities.values() {
            if other.collidable && entity.rect.collide(&other.rect) {
                if entity.rect.get_right() >= other.rect.get_left()
                    && old_rect.get_right() <= other.rect.get_left()
                {
                    entity.rect.set_right(other.rect.get_left());
                } else if entity.rect.get_left() <= other.rect.get_right()
                    && old_rect.get_left() >= other.rect.get_right()
                {
                    entity.rect.set_left(other.rect.get_right());
                }
            }
        }

        entity.rect.pos.y += entity.vel.y * dt;
        let cols = game.tile_map.get_collisions(&entity.rect);
        for col in cols {
            if entity.vel.y > 0.0 {
                entity.rect.pos.y = col.y as f32 - h_size - 0.001;
            } else if entity.vel.y < 0.0 {
                entity.rect.pos.y = col.y as f32 + 1.0 + h_size + 0.001;
            }
        }

        for other in game.entities.values() {
            if other.collidable && entity.rect.collide(&other.rect) {
                if entity.rect.get_bottom() >= other.rect.get_top()
                    && old_rect.get_bottom() <= other.rect.get_top()
                {
                    entity.rect.set_bottom(other.rect.get_top());
                } else if entity.rect.get_top() <= other.rect.get_bottom()
                    && old_rect.get_top() >= other.rect.get_bottom()
                {
                    entity.rect.set_top(other.rect.get_bottom());
                }
            }
        }
    }
}
pub struct ProjectileCollisionComponent {
    owner_id: u32,
}
impl  ProjectileCollisionComponent {
    pub fn new(owner_id: u32) -> Self{
        ProjectileCollisionComponent {owner_id}
    }
}
impl Component for ProjectileCollisionComponent {
    fn update<'a>(&mut self, entity: &mut Entity, game: &mut Game<'a>, dt: f32) {
        entity.rect.pos = entity.rect.pos + entity.vel * dt;

        let mut collided = false;
        for other in game.entities.values() {
            if other.id != self.owner_id && other.rect.collide(&entity.rect) {
                collided = true;
                break;
            }
        }
        if !game.tile_map.get_collisions(&entity.rect).is_empty() {
            collided = true;
        }
        if collided {
            let explosion_sound = game
                .assets
                .load_sound("assets/sounds/explosionCrunch_000.ogg")
                .clone();
            entity.alive = false;
            game.audio_manager.play(explosion_sound).unwrap();
            let images = vec![
                "assets/explosion/explosion1.png",
                "assets/explosion/explosion2.png",
                "assets/explosion/explosion3.png",
                "assets/explosion/explosion4.png",
                "assets/explosion/explosion5.png",
                "assets/explosion/explosion6.png",
                "assets/explosion/explosion7.png",
                "assets/explosion/explosion8.png",
                "assets/explosion/explosion9.png",
                "assets/explosion/explosion10.png",
                "assets/explosion/explosion11.png",
                "assets/explosion/explosion12.png",
            ];
            game.add_entity(Entity::new(
                entity.rect.pos,
                Some(images[0]),
                Vec2::new(0.0, 0.0),
                0.5,
                false,
                vec![Box::new(AnimationComponent {
                    images,
                    time_per_frame: 0.05,
                    cur_time: 0.0,
                })],
            ))
        }
    }
}
