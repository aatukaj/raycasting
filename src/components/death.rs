use super::Component;

pub struct DeathComponent {
    death_sound: &'static str,
}
impl DeathComponent {
    pub fn new(death_sound: &'static str) -> Self {
        DeathComponent {death_sound}
    }
}
impl Component for DeathComponent {
    fn update<'a>(&mut self, entity: &mut crate::entity::Entity<'a>, game: &mut crate::Game<'a>, dt: f32) {
        if entity.health <= 0 {
            entity.alive = false;
            game.audio_manager.play(game.assets.load_sound(self.death_sound, None)).unwrap();
        }
    }
}