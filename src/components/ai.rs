use super::Component;

pub struct BasicAiComponent;

impl Component for BasicAiComponent {
    fn update(&mut self, entity: &mut crate::entity::Entity, game: &mut crate::Game, _dt: f32) {
        entity.vel = (game.entities.get(&0).unwrap().rect.pos - entity.rect.pos).normalize() * 0.5;
        //println!("{:?}, {:?}", entity.pos, entity.rect.pos);
    }
}
