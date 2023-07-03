use crate::{depth_buffer::*, entity::Entity, math::Vec2, Game, FOV};

use super::Component;

pub struct CameraComponent;
impl CameraComponent {
    fn project_entities<'a>(&self, entity: &Entity, game: &mut Game<'a>) {
        let camera_plane = Vec2::new(1.0, 0.0).rotate(entity.look_angle);
        let camera_normal = Vec2::new(camera_plane.y, -camera_plane.x);

        for other in game.entities.values() {
            if let Some(sprite) = other.sprite {
                let enemy_offset_pos = other.rect.pos - entity.rect.pos;
                let enemy_projected_pos =
                    camera_plane * enemy_offset_pos.x + camera_normal * enemy_offset_pos.y;

                let angle = enemy_projected_pos.x.atan2(enemy_projected_pos.y);

                let column = ((angle + FOV) / FOV * game.screen.width as f32) as i32
                    - game.screen.width as i32 / 2;
                if enemy_projected_pos.y > -0.1 && angle.abs() < FOV / 1.5 {
                    game.renderer.data.push(DepthBufferData {
                        distance: enemy_projected_pos.y,
                        column,
                        data_type: BufferDataType::Sprite { surf: sprite },
                    });
                }
            }
        }
    }

    fn cast_rays(&self, entity: &Entity, game: &mut Game) {
        let camera_plane = Vec2::new(1.0, 0.0).rotate(entity.look_angle);
        let camera_dir = Vec2::new(camera_plane.y, -camera_plane.x);
        let ray_start = entity.rect.pos;
        let rays: Vec<Vec2<f32>> = (0..game.screen.width)
            .map(|i| {
                let cam_x = (2 * i) as f32 / game.screen.width as f32 - 1.0;
                camera_dir + camera_plane * cam_x
            })
            .collect();

        for (index, &ray_dir) in rays.iter().enumerate() {
            let ray_unit_step = Vec2::new((1.0 / ray_dir.x).abs(), (1.0 / ray_dir.y).abs());
            let mut map_check = ray_start.as_i32();
            let mut ray_length_1d = Vec2::new(0.0, 0.0);

            let mut step = Vec2::new(0, 0);

            if ray_dir.x < 0.0 {
                step.x = -1;
                ray_length_1d.x = (ray_start.x - map_check.x as f32) * ray_unit_step.x;
            } else {
                step.x = 1;
                ray_length_1d.x = (map_check.x as f32 + 1.0 - ray_start.x) * ray_unit_step.x;
            }
            if ray_dir.y < 0.0 {
                step.y = -1;
                ray_length_1d.y = (ray_start.y - map_check.y as f32) * ray_unit_step.y;
            } else {
                step.y = 1;
                ray_length_1d.y = (map_check.y as f32 + 1.0 - ray_start.y) * ray_unit_step.y;
            }

            let mut tile_found = false;
            let mut direction = Direction::Horizontal;
            let max_steps = 100;
            let mut steps = 0;


            while !tile_found && steps < max_steps {
                if ray_length_1d.x < ray_length_1d.y {
 
                    ray_length_1d.x += ray_unit_step.x;
                    map_check.x += step.x;
                    direction = Direction::Vertical;
                } else {

                    ray_length_1d.y += ray_unit_step.y;
                    map_check.y += step.y;
                    direction = Direction::Horizontal;
                }

                if game.tile_map.get_tile(map_check) == 1 {
                    tile_found = true;
                }
                steps += 1;
            }
            if tile_found {
                

                let distance = match direction {
                    Direction::Horizontal => ray_length_1d.y - ray_unit_step.y,
                    Direction::Vertical => ray_length_1d.x - ray_unit_step.x,
                };
                let intersection = ray_start + ray_dir * distance;

                let percentage = match direction {
                    Direction::Horizontal => intersection.x.fract(),
                    Direction::Vertical => intersection.y.fract(),
                };

                game.renderer.data.push(DepthBufferData {
                    distance,
                    column: index as i32,
                    data_type: BufferDataType::Wall {
                        direction,
                        percentage,
                    },
                });
            }
        }
    }
}

impl Component for CameraComponent {
    fn update<'a>(&mut self, entity: &mut Entity, game: &mut Game, dt: f32) {
        self.cast_rays(entity, game);
        self.project_entities(entity, game);
    }
}
