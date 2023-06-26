use minifb::MouseMode;
use minifb::{Key, Window, WindowOptions};

use std::f32::consts::PI;
use std::time;

const WIDTH: usize = 500;
const HEIGHT: usize = 300;
const TILE_SIZE: usize = 10;
const MAP_WIDTH: usize = WIDTH / 25;
const MAP_HEIGHT: usize = HEIGHT / 25;
const FOV: f32 = PI / 2.5; //90 dgs in radians;

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

    tiles[123..131].iter_mut().for_each(|val| *val = 1);

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

    let mut pos: Vec2<f32> = Vec2::new(0.0, 0.0);
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut m_angle = 0.0;
    let mut now = time::SystemTime::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = now.elapsed().unwrap().as_secs_f32();
        now = time::SystemTime::now();
        screen.fill(0);

        let m_pos = Vec2::from_tuple(window.get_mouse_pos(MouseMode::Clamp).unwrap()) / 2.0;

        let player_tile_pos = pos / TILE_SIZE as f32;
        let m_dir: Vec2<f32> = Vec2::new(0.0, -1.0).rotate(m_angle);
        let ray_start = player_tile_pos.clone();
        let rays: Vec<Vec2<f32>> = (0..WIDTH)
            .map(|i| {
                let a = (i as f32 / WIDTH as f32 - 0.5) * FOV;
                m_dir.rotate(a)
            })
            .collect();

        for y in 0..(HEIGHT / 2) {
            let brigthness = 1.0 - (y as f32 / (HEIGHT / 2) as f32).sqrt();
            let val = (brigthness * 255.0) as u32;
            draw_rect(
                &mut screen,
                Vec2::new(0, y as i32),
                Vec2::new(WIDTH as i32, 1),
                val_from_rgb(val, val, val),
            );
            draw_rect(
                &mut screen,
                Vec2::new(0, (HEIGHT - y) as i32),
                Vec2::new(WIDTH as i32, 1),
                val_from_rgb(val, val, val),
            );
        }

        for (index, &ray_dir) in rays.iter().enumerate() {
            let ray_unit_step = Vec2::new(
                (1.0 + (ray_dir.y / ray_dir.x).powf(2.0)).sqrt(),
                (1.0 + (ray_dir.x / ray_dir.y).powf(2.0)).sqrt(),
            );
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
            let max_distance = 100.0;
            let mut distance = 0.0;
            while !tile_found && distance < max_distance {
                if ray_length_1d.x < ray_length_1d.y {
                    map_check.x += step.x;
                    distance = ray_length_1d.x as f32;
                    ray_length_1d.x += ray_unit_step.x;
                } else {
                    map_check.y += step.y;
                    distance = ray_length_1d.y as f32;
                    ray_length_1d.y += ray_unit_step.y;
                }



                if map_check.x > 0
                    && map_check.x < MAP_WIDTH as i32
                    && map_check.y > 0
                    && map_check.y < MAP_HEIGHT as i32
                {
                    let index = map_check.x + map_check.y * MAP_WIDTH as i32;
                    if tiles[index as usize] == 1 {
                        tile_found = true;
                    }
                }
            }
            if tile_found {
                let intersection = (ray_start + ray_dir * distance) * TILE_SIZE as f32;
                let distance = distance * (-FOV / 2.0 + (FOV / WIDTH as f32) * index as f32).cos();

                let value = (1.0 / distance).min(1.0);
                let height = (value * 1.0 * HEIGHT as f32) as i32;

                draw_rect(
                    &mut screen,
                    Vec2::new(index as i32, HEIGHT as i32 / 2 - height / 2),
                    Vec2::new(1, height),
                    ((value).sqrt() * 255.0).min(255.0) as u32,
                );
                draw_rect(
                    &mut screen,
                    intersection.as_i32() - Vec2::new(4, 4),
                    Vec2::new(7, 7),
                    0xFFFF,
                );
            }
        }

        window.get_keys().iter().for_each(|key| match key {
            Key::A => pos = pos + Vec2::new(-50.0, 0.0).rotate(m_angle) * dt,
            Key::D => pos = pos + Vec2::new(50.0, 0.0).rotate(m_angle) * dt,
            Key::W => pos = pos + Vec2::new(0.0, -50.0).rotate(m_angle) * dt,
            Key::S => pos = pos + Vec2::new(0.0, 50.0).rotate(m_angle) * dt,
            Key::Left => m_angle -= 2.0 * dt,
            Key::Right => m_angle += 2.0 * dt,
            _ => (),
        });

        for (i, &val) in tiles.iter().enumerate() {
            if val == 1 {
                draw_rect(
                    &mut screen,
                    Vec2::new(
                        (i % MAP_WIDTH * TILE_SIZE) as i32,
                        (i / MAP_WIDTH * TILE_SIZE) as i32,
                    ),
                    Vec2::new(TILE_SIZE as i32, TILE_SIZE as i32),
                    0xFF,
                )
            }
        }
        for x in 0..MAP_WIDTH {
            draw_rect(
                &mut screen,
                Vec2::new((x * TILE_SIZE) as i32, 0),
                Vec2::new(1, (TILE_SIZE * MAP_HEIGHT) as i32),
                0xffffff,
            );
        }
        for y in 0..MAP_HEIGHT {
            draw_rect(
                &mut screen,
                Vec2::new(0, (y * TILE_SIZE) as i32),
                Vec2::new((TILE_SIZE * MAP_WIDTH) as i32, 1),
                0xffffff,
            );
        }
        screen.blit(&image, pos.x.floor() as i32, pos.y.floor() as i32);
        window
            .update_with_buffer(&screen.pixel_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
