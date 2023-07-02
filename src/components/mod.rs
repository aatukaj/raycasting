use crate::{entity::Entity, Game};




pub trait Component {
    fn update(&self, entity: &mut Entity, game: &mut Game, dt: f32);
}

pub mod collision;
pub use collision::*;
pub mod camera;
pub use camera::*;
pub mod input;
pub use input::*;
pub mod ai;
pub use ai::*;