use crate::{entity::Entity, Game};

pub trait Component {
    fn update<'a>(&mut self, entity: &mut Entity<'a>, game: &mut Game<'a>, dt: f32);
}

pub mod collision;
pub use collision::*;
pub mod camera;
pub use camera::*;
pub mod input;
pub use input::*;
pub mod ai;
pub use ai::*;
pub mod anim;
pub use anim::*;
pub mod death;
pub use death::*;
