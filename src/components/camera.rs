use super::Component;
use crate::tile_map::*;
use crate::{depth_buffer::*, entity::Entity, Game};
use glam::*;
pub struct CameraComponent;
impl CameraComponent {
    fn project_entities<'a>(
        &self,
        entity: &Entity,
        game: &mut Game<'a>,
        camera_plane: Vec2,
        camera_normal: Vec2,
    ) {
        let transform_mat = Mat2::from_cols(camera_plane, camera_normal);
        for other in game.entities.values() {
            if let Some(sprite) = other.sprite {
                let enemy_offset_pos = other.rect.pos - entity.rect.pos;
                let enemy_projected_pos = transform_mat.mul_vec2(enemy_offset_pos);

                let column = ((1.0 + enemy_projected_pos.x / enemy_projected_pos.y)
                    * (game.screen.width as f32 / 2.0)) as i32;
                if enemy_projected_pos.y > -0.1 {
                    game.renderer.data.push(DepthBufferData {
                        distance: enemy_projected_pos.y,
                        column,
                        data_type: BufferDataType::Sprite { surf: sprite },
                    });
                }
            }
        }
    }

    fn cast_rays(&self, entity: &Entity, game: &mut Game, camera_plane: Vec2, camera_normal: Vec2) {
        let ray_start = entity.rect.pos;
        let rays: Vec<Vec2> = (0..game.screen.width)
            .map(|i| {
                let cam_x = (2 * i) as f32 / game.screen.width as f32 - 1.0;
                camera_normal + camera_plane * cam_x
            })
            .collect();

        for (index, &ray_dir) in rays.iter().enumerate() {
            let ray_unit_step = Vec2::new((1.0 / ray_dir.x).abs(), (1.0 / ray_dir.y).abs());
            let mut map_check = ray_start.as_ivec2();
            let mut ray_length_1d = Vec2::new(0.0, 0.0);

            let mut step = IVec2::new(0, 0);

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
            let mut direction;
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
                let tile = game.tile_map.get_tile(map_check);
                if let Some(tile) = tile {
                    let mut distance = match direction {
                        Direction::Horizontal => ray_length_1d.y - ray_unit_step.y,
                        Direction::Vertical => ray_length_1d.x - ray_unit_step.x,
                    };
                    if tile.tile_type == TileType::Wall {}
                    match &tile.tile_type {
                        TileType::Wall => tile_found = true,
                        TileType::Door(open_amount, door_dir) => {
                            if &direction == door_dir {
                                let percentage = (ray_start + ray_dir * distance).fract();
                                let step = ray_unit_step * 0.5;
                                distance += match direction {
                                    Direction::Horizontal
                                        if percentage.x + step.y * ray_dir.x
                                            < 1.0 - open_amount =>
                                    {
                                        step.y
                                    }
                                    Direction::Vertical
                                        if percentage.y + step.x * ray_dir.y
                                            < 1.0 - open_amount =>
                                    {
                                        step.x
                                    }

                                    _ => continue,
                                };
                            } else {
                                continue;
                            }
                        }
                        TileType::Subwall(offset, wall_dir) => {
                            if &direction == wall_dir {
                                let step = ray_unit_step * *offset;
                                distance += match direction {
                                    Direction::Horizontal => step.y,
                                    Direction::Vertical => step.x,
                                };
                            } else {
                                continue;
                            }
                        }
                    }

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
                            sprite: tile.sprites[match direction {
                                Direction::Horizontal => 0,
                                Direction::Vertical => 1,
                            }],
                        },
                    });
                }
                steps += 1;
            }
        }
    }
}

impl Component for CameraComponent {
    fn update<'a>(&mut self, entity: &mut Entity, game: &mut Game, _dt: f32) {
        let camera_plane = Vec2::new(1.0, 0.0).rotate(Vec2::from_angle(entity.look_angle));
        let camera_normal = Vec2::new(camera_plane.y, -camera_plane.x);
        self.cast_rays(entity, game, camera_plane, camera_normal);
        self.project_entities(entity, game, camera_plane, camera_normal);
    }
}
