use crate::{entity::Entity, Game};

use super::Component;
pub struct BasicCollisionComponent;
impl Component for BasicCollisionComponent {
    fn update(&self, entity: &mut Entity, game: &mut Game, dt: f32) {
        let old_rect = entity.rect.clone();

        entity.rect.pos.x += entity.vel.x * dt;
        let h_size = entity.rect.size / 2.0;
        let cols = game.tile_map.get_collisions(&entity.rect);
        for col in cols {
            if entity.vel.x > 0.0 {
                entity.rect.pos.x = col.x as f32 - h_size - 0.00420; //i dont like the arbitrary subtraction but it fixes a bug
            } else if entity.vel.x < 0.0 {
                entity.rect.pos.x = col.x as f32 + 1.0 + h_size;
            }
        }


        for other in game.entities.values() {
            if other.collidable && entity.rect.collide(&other.rect) {
                if entity.rect.get_right() >= other.rect.get_left() && old_rect.get_right() <= other.rect.get_left(){
                    entity.rect.set_right(other.rect.get_left());
                } else if entity.rect.get_left() <= other.rect.get_right() && old_rect.get_left() >= other.rect.get_right() {
                    entity.rect.set_left(other.rect.get_right());
                }
            }
        }

        entity.rect.pos.y += entity.vel.y * dt;
        let cols = game.tile_map.get_collisions(&entity.rect);
        for col in cols {
            if entity.vel.y > 0.0 {
                entity.rect.pos.y = col.y as f32 - h_size - 0.00420;
            } else if entity.vel.y < 0.0 {
                entity.rect.pos.y = col.y as f32 + 1.0 + h_size;
            }
        }

        for other in game.entities.values() {
            if other.collidable && entity.rect.collide(&other.rect) {
                if entity.rect.get_bottom() >= other.rect.get_top() && old_rect.get_bottom() <= other.rect.get_top(){
                    entity.rect.set_bottom(other.rect.get_top());
                    
                } else if entity.rect.get_top() <= other.rect.get_bottom() && old_rect.get_top() >= other.rect.get_bottom() {
                    entity.rect.set_top(other.rect.get_bottom());
                }
            }
        }
    }
}
pub struct ProjectileCollisionComponent;
impl Component for ProjectileCollisionComponent {
    fn update(&self, entity: &mut Entity, game: &mut Game, dt: f32) {
        entity.rect.pos = entity.rect.pos + entity.vel * dt;
        let explosion_sound = game
            .assets
            .load_sound("assets/sounds/explosionCrunch_000.ogg")
            .clone();

        for other in game.entities.values() {
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
