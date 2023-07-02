use super::Component;

pub struct BasicAiComponent;

impl Component for BasicAiComponent {
    fn update(&self, entity: &mut crate::entity::Entity, game: &mut crate::Game, dt: f32) {
        entity.vel = (game.entities[0].pos - entity.pos).normalize() * 0.1;
        //println!("{:?}, {:?}", entity.pos, entity.rect.pos);
    }
}
