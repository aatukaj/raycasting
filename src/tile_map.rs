use std::{error::Error, fs};

use crate::{rect::Rect};
use glam::*;
pub struct TileMap {
    width: usize,
    height: usize,
    buf: Vec<u8>,
}
impl TileMap {
    pub fn get_tile(&self, pos: IVec2) -> u8 {
        if 0 <= pos.x && pos.x < self.width as i32 && 0 <= pos.y && pos.y < self.height as i32 {
            return self.buf[pos.x as usize + pos.y as usize * self.width];
        }
        0
    }
    pub fn get_collisions<'a>(&'a self, rect: &'a Rect) -> Vec<IVec2> {
        rect.get_corners()
            .iter()
            .map(|pos| pos.as_ivec2())
            .filter(|pos| self.get_tile(*pos) != 0)
            .collect()
    }
}

pub fn load_map(path: &str) -> Result<TileMap, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let lines: Vec<&str> = contents.lines().collect();
    let width = lines[0].len();
    let height = lines.len();
    let mut buf = Vec::with_capacity(width * height);
    for line in lines {
        for c in line.chars(){

            buf.push(match c {
                '1' => 1,
                '2' => 2,
                _ => 0,
            })
        }
    }

    Ok(TileMap { width, height, buf })
}
