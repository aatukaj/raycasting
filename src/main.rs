use minifb::MouseMode;
use minifb::{Key, Window, WindowOptions};

use std::error::Error;
use std::fs;
use std::time;

const WIDTH: usize = 500;
const HEIGHT: usize = 300;

struct Surface {
    width: usize,
    height: usize,
    pixel_buffer: Vec<u32>,
}

struct LineDrawer {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    dx: i32,
    dy: i32,
    x: i32,
    y: i32,
    d: i32,
}
impl LineDrawer {
    fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        let dx = x1 - x0;
        let dy = y1 - y0;
        LineDrawer {
            x0,
            x1,
            y0,
            y1,
            dx,
            dy,
            x: x0,
            y: y0,
            d: 2 * dy - dx,
        }
    }
}

impl Iterator for LineDrawer {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        self.x += 1;
        if self.x > self.x1 {
            return None;
        }
        if self.d > 0 {
            self.y += 1;
            self.d -= 2 * self.dx;
        }
        self.d += 2*self.dy;
        Some((self.x, self.y))
    }
}

impl Surface {
    fn empty(width: usize, height: usize) -> Self {
        Surface {
            width,
            height,
            pixel_buffer: vec![0; width * height],
        }
    }

    fn fill(&mut self, value: u32) {
        self.pixel_buffer.fill(value);
    }

    fn set_pixel(&mut self, x: u32, y: u32, value: u32) -> Result<(), ()> {
        let index = x as usize + y as usize * self.width;
        if index < self.pixel_buffer.len() {
            self.pixel_buffer[index] = value;
            return Ok(());
        }
        return Err(());
    }

    fn blit(&mut self, source: &Surface, x: usize, y: usize) {
        for (i, &val) in source.pixel_buffer.iter().enumerate() {
            let x = i % source.width + x;
            let y = i / source.width + y;
            let index = x + y * self.width;
            if index < self.pixel_buffer.len() {
                self.pixel_buffer[index] = val;
            }
        }
    }
}

struct Vec2 {
    x: f32,
    y: f32,
}
impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
}

fn bytes_to_u32(bytes: &[u8]) -> Result<u32, &str> {
    match bytes.len() {
        3 => Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0])),
        4 => Ok(u32::from_le_bytes(bytes.try_into().unwrap())),

        _ => Err("len of `bytes` must be 3 or 4"),
    }
}

fn load_bmp(path: &str) -> Result<Surface, Box<dyn Error>> {
    //doesnt work on non rgba bmps

    let px_offset_index = 0x0A;
    let img_size_index = 0x12;

    let file = fs::read(path)?;
    let pixel_array_offset = bytes_to_u32(&file[px_offset_index..px_offset_index + 4])? as usize;
    let width = bytes_to_u32(&file[img_size_index..img_size_index + 4])? as usize;
    let height = bytes_to_u32(&file[img_size_index + 4..img_size_index + 8])? as usize;

    //this could propably be improved

    let mut pixel_buffer: Vec<u32> = vec![0; width * height];
    for (i, value) in file[pixel_array_offset..]
        .chunks_exact(4)
        .map(|bytes| bytes_to_u32(bytes).unwrap())
        .enumerate()
    {
        let x = i % width;
        let y = height - 1 - i / width;
        pixel_buffer[x + y * width] = value;
    }

    Ok(Surface {
        width: width as usize,
        height: height as usize,
        pixel_buffer,
    })
}

fn draw_line(surf: &mut Surface, x0: i32, y0: i32, x1: i32, y1: i32, value: u32) {
    if (y1 - y0).abs() < (x1 - x0).abs() {
        if x0 > x1 {
            draw_line_low(surf, x1, y1, x0, y0, value);
        } else {
            draw_line_low(surf, x0, y0, x1, y1, value);
        }
    } else {
        if y0 > y1 {
            draw_line_high(surf, x1, y1, x0, y0, value);
        } else {
            draw_line_high(surf, x0, y0, x1, y1, value);
        }
    }
}

fn draw_line_low(surf: &mut Surface, x0: i32, y0: i32, x1: i32, y1: i32, value: u32) {
    let dx = x1 - x0;
    let mut dy = y1 - y0;
    let mut yi = 1;
    if dy < 0 {
        yi = -1;
        dy = -dy;
    }

    let mut d = 2 * dy - dx;
    let mut y = y0;

    for x in x0..=x1 {
        surf.set_pixel(x as u32, y as u32, value);

        if d > 0 {
            y += yi;
            d += 2 * (dy - dx);
        } else {
            d += 2 * dy;
        }
    }
}
fn draw_line_high(surf: &mut Surface, x0: i32, y0: i32, x1: i32, y1: i32, value: u32) {
    let mut dx = x1 - x0;
    let dy = y1 - y0;
    let mut xi = 1;
    if dx < 0 {
        xi = -1;
        dx = -dx;
    }

    let mut d = 2 * dx - dy;
    let mut x = x0;

    for y in y0..=y1 {
        surf.set_pixel(x as u32, y as u32, value);

        if d > 0 {
            x += xi;
            d += 2 * (dx - dy);
        } else {
            d += 2 * dx;
        }
    }
}

fn main() {
    let mut screen = Surface::empty(WIDTH, HEIGHT);
    let image = load_bmp("assets/brick.bmp").expect("couldnt load");

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH * 3,
        HEIGHT * 3,
        WindowOptions {
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            ..Default::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut pos = Vec2::new(0.0, 0.0);
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut now = time::SystemTime::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = now.elapsed().unwrap().as_secs_f32();
        now = time::SystemTime::now();
        screen.fill(0);
        screen.blit(&image, pos.x.round() as usize, pos.y.round() as usize);
        let m_pos = window.get_mouse_pos(MouseMode::Clamp).unwrap();
        draw_line(
            &mut screen,
            m_pos.0 as i32 / 3,
            m_pos.1 as i32 / 3,
            pos.x as i32,
            pos.y as i32,
            0xFFFFFF,
        );

        window.get_keys().iter().for_each(|key| match key {
            Key::A => pos.x -= 40.0 * dt,
            Key::D => pos.x += 40.0 * dt,
            Key::W => pos.y -= 40.0 * dt,
            Key::S => pos.y += 40.0 * dt,
            _ => (),
        });
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&screen.pixel_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
