use glam::{Vec2, vec2};

use crate::{entity::Entity, Game};

use super::{AnimationComponent, Component};
pub struct BasicCollisionComponent;
impl Component for BasicCollisionComponent {
    fn update<'a>(&mut self, entity: &mut Entity, game: &mut Game, dt: f32) {
        let old_rect = entity.rect.clone();

        entity.rect.pos.x += entity.vel.x * dt;

        let cols = game.tile_map.get_collisions(&entity.rect);

        for col in game
            .entities
            .values()
            .filter_map(|other| {
                let rect = other.rect;
                (entity.rect.collide(&rect) && other.collidable).then_some(rect)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .chain(cols.into_iter())
        {
            {
                if entity.rect.get_right() >= col.get_left()
                    && old_rect.get_right() <= col.get_left()
                {
                    entity.rect.set_right(col.get_left() - 0.001);
                } else if entity.rect.get_left() <= col.get_right()
                    && old_rect.get_left() >= col.get_right()
                {
                    entity.rect.set_left(col.get_right() + 0.001);
                }
            }
        }

        entity.rect.pos.y += entity.vel.y * dt;
        let cols = game.tile_map.get_collisions(&entity.rect);

        for col in game
            .entities
            .values()
            .filter_map(|other| {
                let rect = other.rect;
                (entity.rect.collide(&rect) && other.collidable).then_some(rect)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .chain(cols.into_iter())
        {
            {
                if entity.rect.get_bottom() >= col.get_top()
                    && old_rect.get_bottom() <= col.get_top()
                {
                    entity.rect.set_bottom(col.get_top() - 0.001);
                } else if entity.rect.get_top() <= col.get_bottom()
                    && old_rect.get_top() >= col.get_bottom()
                {
                    entity.rect.set_top(col.get_bottom() + 0.001);
                }
            }
        }
    }
}
pub struct ProjectileCollisionComponent {
    owner_id: u32,
}
impl ProjectileCollisionComponent {
    pub fn new(owner_id: u32) -> Self {
        ProjectileCollisionComponent { owner_id }
    }
}
impl Component for ProjectileCollisionComponent {
    fn update<'a>(&mut self, entity: &mut Entity, game: &mut Game<'a>, dt: f32) {
        let mut new_rect = entity.rect.clone();
        new_rect.pos = entity.rect.pos + entity.vel * dt;

        let mut collided = false;
        for other in game.entities.values() {
            if other.id != self.owner_id && other.rect.collide(&new_rect) {
                collided = true;
                break;
            }
        }
        if !game.tile_map.get_collisions(&new_rect).is_empty() {
            collided = true;
        }
        if collided {
            let explosion_sound = game
                .assets
                .load_sound("assets/sounds/explosionCrunch_000.ogg", None)
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
                vec2(0.0, 0.0),
                1.0,
                false,
                vec![Box::new(AnimationComponent {
                    images,
                    time_per_frame: 0.05,
                    cur_time: 0.0,
                }), Box::new(BasicCollisionComponent)],
            ))
        }
        entity.rect = new_rect;
    }
}
