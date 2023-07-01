use std::{error::Error, fs};

use crate::{math::Vec2, rect::Rect};

pub struct TileMap {
    width: usize,
    height: usize,
    buf: Vec<u8>,
}
impl TileMap {
    pub fn get_tile(&self, pos: Vec2<i32>) -> u8 {
        if 0 <= pos.x && pos.x < self.width as i32 && 0 <= pos.y && pos.y < self.height as i32 {
            return self.buf[pos.x as usize + pos.y as usize * self.width];
        }
        0
    }
    pub fn get_collisions<'a>(&'a self, rect: &'a Rect) -> Vec<Vec2<i32>> {
        rect.get_corners()
            .iter()
            .map(|pos| pos.as_i32())
            .filter(|pos| self.get_tile(*pos) != 0)
            .collect()
    }
}

pub fn load_map(path: &str) -> Result<TileMap, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let lines: Vec<&str> = contents.split_ascii_whitespace().collect();
    let width = lines[0].len();
    let height = lines.len();
    let mut buf = Vec::with_capacity(width * height);
    for line in lines {
        for c in line.chars() {
            buf.push(c.to_digit(10).unwrap() as u8)
        }
    }
    Ok(TileMap { width, height, buf })
}
