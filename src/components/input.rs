use minifb::Key;

use crate::{entity::Entity, Game};
use glam::Vec2;
use super::{Component, ProjectileCollisionComponent};

pub struct PlayerInputComponent;
impl Component  for PlayerInputComponent {
    fn update<'a>(&mut self, entity: &mut Entity<'a>, game: &mut Game, dt: f32) {
        let mut vel = Vec2::new(0.0, 0.0);
        let player = entity;
        game.window.get_keys().iter().for_each(|key| match key {
            Key::A => vel += Vec2::new(-5.0, 0.0).rotate(Vec2::from_angle(player.look_angle)),
            Key::D => vel += Vec2::new(5.0, 0.0).rotate(Vec2::from_angle(player.look_angle)),
            Key::W => vel += Vec2::new(0.0, -5.0).rotate(Vec2::from_angle(player.look_angle)),
            Key::S => vel += Vec2::new(0.0, 5.0).rotate(Vec2::from_angle(player.look_angle)),
            Key::Left => player.look_angle -= 2.0 * dt,
            Key::Right => player.look_angle += 2.0 * dt,

            _ => (),
        });

        player.vel = vel;
        game.window
            .get_keys_pressed(minifb::KeyRepeat::No)
            .iter()
            .for_each(|key| match key {
                Key::Space => {
                    let dir = Vec2::new(0.0, -1.0).rotate(Vec2::from_angle(player.look_angle));

                    game.add_entity(Entity::new(
                        player.rect.pos + dir * 0.5,
                        Some("assets/explosion/explosion1.png"),
                        dir * 8.0,
                        0.3,
                        false,
                        vec![Box::new(ProjectileCollisionComponent::new(player.id))],
                    ));
                    let sound_data = game.assets.load_sound("assets/sounds/laserRetro_002.ogg");
                    game.audio_manager.play(sound_data.clone()).unwrap();
                }
                _ => (),
            })
    }
}
