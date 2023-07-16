use std::{error::Error, fs};

use crate::{depth_buffer::Direction, rect::Rect};
use glam::*;
pub struct TileMap<'a> {
    pub width: usize,
    pub height: usize,
    buf: Vec<Option<Tile<'a>>>,
    pub tile_update_indeces: Vec<usize>,
}
impl<'a> TileMap<'a> {
    pub fn get_tile(&self, pos: IVec2) -> Option<&Tile<'a>> {
        if 0 <= pos.x && pos.x < self.width as i32 && 0 <= pos.y && pos.y < self.height as i32 {
            return self.buf[pos.x as usize + pos.y as usize * self.width].as_ref();
        }
        None
    }
    pub fn get_tile_mut(&mut self, pos: IVec2) -> Option<&mut Tile<'a>> {
        if 0 <= pos.x && pos.x < self.width as i32 && 0 <= pos.y && pos.y < self.height as i32 {
            return self.buf[pos.x as usize + pos.y as usize * self.width].as_mut();
        }
        None
    }
    pub fn update(&mut self, dt: f32) {
        let mut new_indeces=  Vec::new();
        for &index in &self.tile_update_indeces {
            if let Some(ref mut tile) = self.buf[index] {
                if let TileType::Door(ref mut percentage, _) = tile.tile_type {
                    if *percentage >= 1.0 {
                        continue;
                    }
                    *percentage += dt / 3.0;
                    new_indeces.push(index);
                }

            }
        }
        self.tile_update_indeces = new_indeces;
    }
    pub fn get_collisions(&'a self, rect: &'a Rect) -> Vec<Rect> {
        rect.get_corners()
            .iter()
            .filter_map(|pos| {
                if let Some(tile) = self.get_tile(pos.as_ivec2()) {
                    let pos = pos.floor() + vec2(0.5, 0.5);
                    let tile_rect = match tile.tile_type {
                        TileType::Wall => Rect {
                            pos,
                            width: 1.0,
                            height: 1.0,
                        },
                        TileType::Subwall(offset, direction) => match direction {
                            Direction::Horizontal => Rect {
                                pos,
                                width: 1.0,
                                height: (1.0 - offset).min(0.2),
                            },
                            Direction::Vertical => Rect {
                                pos,
                                width: (1.0 - offset).min(0.2),
                                height: 1.0,
                            },
                        },
                        TileType::Door(opened, direction) => match direction {
                            Direction::Horizontal => Rect {
                                pos: pos - vec2(opened, 0.0),
                                width: 1.0 * (1.0 - opened),
                                height: 0.2,
                            },
                            Direction::Vertical => Rect {
                                pos: pos - vec2(0.0, opened),
                                width: 0.2,
                                height: 1.0 * (1.0 - opened),
                            },
                        },
                    };
                    rect.collide(&tile_rect).then_some(tile_rect)
                } else {
                    None
                }
            })
            .collect()
    }
}

pub struct Tile<'a> {
    pub tile_type: TileType,
    pub projectile_passable: bool,
    pub sprites: [&'a str; 2],
}
#[derive(PartialEq)]
pub enum TileType {
    Wall,
    Subwall(f32, Direction),
    Door(f32, Direction),
}

pub fn load_map(path: &str) -> Result<TileMap, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let width = lines[0].len();
    let height = lines.len();
    let mut buf = Vec::with_capacity(width * height);
    for line in lines {
        for c in line.chars() {
            buf.push(match c {
                '1' => Some(Tile {
                    tile_type: TileType::Wall,
                    projectile_passable: false,
                    sprites: ["assets/bricksmall.png", "assets/bricksmall2.png"],
                }),
                '2' => Some(Tile {
                    tile_type: TileType::Wall,
                    projectile_passable: false,
                    sprites: ["assets/white.png", "assets/white.png"],
                }),
                '=' => Some(door(Direction::Horizontal)),
                '/' => Some(door(Direction::Vertical)),
                '-' => Some(subwall(Direction::Horizontal)),
                '|' => Some(subwall(Direction::Vertical)),
                'w' => Some(wood(Direction::Vertical)),
                'W' => Some(wood(Direction::Horizontal)),
                _ => None,
            })
        }
    }

    Ok(TileMap {
        width,
        height,
        buf,
        tile_update_indeces: Vec::new(),
    })
}
fn door<'a>(direction: Direction) -> Tile<'a> {
    Tile {
        tile_type: TileType::Door(0.0, direction),
        projectile_passable: false,
        sprites: ["assets/door.png", "assets/door.png"],
    }
}
fn subwall<'a>(direction: Direction) -> Tile<'a> {
    Tile {
        tile_type: TileType::Subwall(0.5, direction),
        projectile_passable: false,
        sprites: ["assets/bars.png", "assets/bars.png"],
    }
}
fn wood<'a>(direction: Direction) -> Tile<'a> {
    Tile {
        tile_type: TileType::Subwall(0.3, direction),
        projectile_passable: false,
        sprites: ["assets/wood.png", "assets/wood.png"],
    }
}
