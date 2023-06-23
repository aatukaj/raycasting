use minifb::MouseMode;
use minifb::{Key, Window, WindowOptions};

use std::time;

const WIDTH: usize = 500;
const HEIGHT: usize = 300;
const TILE_SIZE: usize = 25;
const MAP_WIDTH: usize = WIDTH / TILE_SIZE;
const MAP_HEIGHT: usize = HEIGHT / TILE_SIZE;

mod math;
use math::*;
mod surface;
use surface::*;
mod drawing;
use drawing::*;
mod file;
use file::*;

fn main() {
    let mut screen = Surface::empty(WIDTH, HEIGHT);
    let image = load_bmp("assets/player.bmp").expect("couldnt load");
    let mut tiles = vec![0u8; WIDTH * HEIGHT];
    tiles[25..30].iter_mut().for_each(|val| *val = 1);
    tiles[63..68].iter_mut().for_each(|val| *val = 1);

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH * 2,
        HEIGHT * 2,
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
        println!("{:?}", pos);

        let dt = now.elapsed().unwrap().as_secs_f32();
        now = time::SystemTime::now();
        screen.fill(0);
        for (i, &val) in tiles.iter().enumerate() {
            if val == 1 {
                draw_rect(
                    &mut screen,
                    (i % MAP_WIDTH * TILE_SIZE) as i32,
                    (i / MAP_WIDTH * TILE_SIZE) as i32,
                    TILE_SIZE as i32,
                    TILE_SIZE as i32,
                    0xFF,
                )
            }
        }
        for x in 0..MAP_WIDTH {
            draw_line(
                &mut screen,
                (x * TILE_SIZE) as i32,
                0,
                (x * TILE_SIZE) as i32,
                HEIGHT as i32,
                0xffffff,
            );
        }
        for y in 0..MAP_HEIGHT {
            draw_line(
                &mut screen,
                0,
                (y * TILE_SIZE) as i32,
                WIDTH as i32,
                (y * TILE_SIZE) as i32,
                0xffffff,
            );
        }

        let m_pos = window.get_mouse_pos(MouseMode::Clamp).unwrap();
        draw_line(
            &mut screen,
            m_pos.0 as i32 / 2,
            m_pos.1 as i32 / 2,
            pos.x as i32 + (image.width / 2) as i32,
            pos.y as i32 + (image.height / 2) as i32,
            0xFF0000,
        );
        screen.blit(&image, pos.x.floor() as usize, pos.y.floor() as usize);

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
