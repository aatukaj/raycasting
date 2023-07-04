use crate::{
    drawing::{draw_rect, val_from_rgb},
    math::set_value_brightness,
    surface::Surface,
    AssetCache,
};
use glam::*;
use std::{cmp::Ordering, collections::BinaryHeap};

pub enum Direction {
    Horizontal,
    Vertical,
}

pub struct DepthBufferData<'a> {
    pub distance: f32,
    pub column: i32,
    pub data_type: BufferDataType<'a>,
}

impl Ord for DepthBufferData<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.total_cmp(&other.distance)
    }
}
impl PartialOrd for DepthBufferData<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for DepthBufferData<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl Eq for DepthBufferData<'_> {}

pub enum BufferDataType<'a> {
    Wall {
        direction: Direction,
        percentage: f32,
        wall_type: u8,
    },
    Sprite {
        surf: &'a str,
    },
}

pub struct DepthBufferRenderer<'a> {
    pub data: BinaryHeap<DepthBufferData<'a>>,
}
impl DepthBufferRenderer<'_> {
    pub fn new(capacity: usize) -> Self {
        DepthBufferRenderer {
            data: BinaryHeap::with_capacity(capacity),
        }
    }
    pub fn render(&mut self, screen: &mut Surface, sprites: &mut AssetCache) {
        for _ in 0..self.data.len() {
            let buf_data: DepthBufferData<'_> = self.data.pop().unwrap();
            let value = 1.0 / buf_data.distance;

            let brightness = (value.sqrt() + 0.2).min(1.0);
            match buf_data.data_type {
                BufferDataType::Wall {
                    direction,
                    percentage,
                    wall_type,
                } => {
                    let wall_tex = sprites.load_png(match (direction, wall_type) {
                        (Direction::Horizontal, 1) => "assets/bricksmall.png",
                        (Direction::Vertical, 1) => "assets/bricksmall2.png",
                        (_, 2) => "assets/door.png",
                        _ => "assets/debug.png",
                    });

                    let height = (value * 1.0 * screen.height as f32) as i32;
                    let scale = height as f32 / wall_tex.height as f32;

                    let offset = screen.height as i32 / 2 - height / 2;
                    let wall_x = (wall_tex.width as f32 * percentage) as usize;
                    let x = buf_data.column as u32;
                    for y in 0..height {
                        let col = wall_tex.pixel_buffer
                            [wall_x + (y as f32 / scale) as usize * wall_tex.width];
                        if col != 0 {
                            let _ = screen.set_pixel(
                                x,
                                (y + offset) as u32,
                                set_value_brightness(col, brightness),
                            );
                        }
                        
                    }
                }
                BufferDataType::Sprite { surf } => {
                    screen.blit_scaled(
                        sprites.load_png(surf),
                        IVec2::new(buf_data.column, screen.height as i32 / 2),
                        1.0 / buf_data.distance * 16.0,
                    );
                }
            }
        }
        self.data.clear();
    }
}
