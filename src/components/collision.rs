
use crate::{entity::Entity, Game};

use super::Component;
pub struct BasicCollisionComponent;
impl Component for BasicCollisionComponent {
    fn update(&self, entity: &mut Entity, game: &mut Game, dt: f32) {
        entity.rect.pos = entity.pos;
        entity.rect.pos.x += entity.vel.x * dt;
        let h_size = entity.rect.size / 2.0;
        let cols = game.tile_map.get_collisions(&entity.rect);
        for col in cols {
            if entity.vel.x > 0.0 {
                entity.rect.pos.x = col.x as f32 - h_size - 0.00420; //i dont like the arbitrary subtraction but it fixes a bug
            } else {
                entity.rect.pos.x = col.x as f32 + 1.0 + h_size;
            }
        }
        for other in game.entities.iter() {
            if other.collidable && entity.rect.collide(&other.rect) {
                println!("yeppers");
                if entity.vel.x > 0.0 {
                    entity.rect.set_right(other.rect.get_left() - 0.01);
                } else {
                    entity.rect.set_left(other.rect.get_right() + 0.01);
                }
            }
        }

        entity.rect.pos.y += entity.vel.y * dt;
        let cols = game.tile_map.get_collisions(&entity.rect);
        for col in cols {
            if entity.vel.y > 0.0 {
                entity.rect.pos.y = col.y as f32 - h_size - 0.00420;
            } else {
                entity.rect.pos.y = col.y as f32 + 1.0 + h_size;
            }
        }
        
        for other in game.entities.iter() {
            
            if other.collidable && entity.rect.collide(&other.rect) {
                if entity.vel.y > 0.0 {
                    entity.rect.set_bottom(other.rect.get_top() - 0.01);
                } else {
                    entity.rect.set_top(other.rect.get_bottom() + 0.01);
                }
            }
        }
        entity.pos = entity.rect.pos;
    }
}
pub struct ProjectileCollisionComponent;
impl Component for ProjectileCollisionComponent {
    fn update(&self, entity: &mut Entity, game: &mut Game, dt: f32) {
        entity.pos = entity.pos + entity.vel * dt;
        entity.rect.pos = entity.pos;
        let explosion_sound = game
            .assets
            .load_sound("assets/sounds/explosionCrunch_000.ogg")
            .clone();
        println!("{}", game.entities.len());
        for other in &game.entities {
            if other.rect.collide(&entity.rect) {
                entity.alive = false;
                game.audio_manager.play(explosion_sound).unwrap();
                return;
            }
        }
        if !game.tile_map.get_collisions(&entity.rect).is_empty() {
            entity.alive = false;
            game.audio_manager.play(explosion_sound).unwrap();
        }
    }
}