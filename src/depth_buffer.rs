use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
};

use crate::{
    drawing::{draw_rect, val_from_rgb},
    math::Vec2,
    surface::Surface,
    AssetCache,
};

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
            let value = (1.0 / buf_data.distance).min(1.0);
            match buf_data.data_type {
                BufferDataType::Wall {
                    direction,
                    percentage,
                } => {
                    let wall_tex = sprites.load(match direction {
                        Direction::Horizontal => "assets/bricksmall.bmp",
                        Direction::Vertical => "assets/bricksmall2.bmp"
                    });

                    let height = (value * 1.5 * screen.height as f32) as i32;
                    let scale = height as f32 / wall_tex.height as f32;

                    let offset = screen.height as i32 / 2 - height / 2;
                    let wall_x = (wall_tex.width as f32 * percentage) as usize;
                    let x = buf_data.column as u32;
                    for y in 0..height {
                        let value = wall_tex.pixel_buffer
                            [wall_x + (y as f32 / scale) as usize * wall_tex.width];
                        screen.set_pixel(x, (y + offset) as u32, value);
                    }

                    /*
                    let value = (value).sqrt();
                    let col = match direction {
                        Direction::Horizontal => val_from_rgb(
                            (0x12 as f32 * value) as u32,
                            (0x4e as f32 * value) as u32,
                            (0x89 as f32 * value) as u32,
                        ),
                        Direction::Vertical => val_from_rgb(
                            (0x63 as f32 * value) as u32,
                            (0xc7 as f32 * value) as u32,
                            (0x4d as f32 * value) as u32,
                        ),
                    };
                    draw_rect(
                        screen,
                        Vec2::new(buf_data.column, screen.height as i32 / 2 - height / 2),
                        Vec2::new(1, height),
                        col,
                    );
                    */
                }
                BufferDataType::Sprite { surf } => {
                    screen.blit_scaled(
                        sprites.load(surf),
                        Vec2::new(buf_data.column, screen.height as i32 / 2),
                        1.0 / buf_data.distance * 20.0,
                    );
                }
            }
        }
        self.data.clear();
    }
}
