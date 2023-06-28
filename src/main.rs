use minifb::{Key, MouseMode, Window, WindowOptions};
use std::collections::binary_heap::*;

use std::f32::consts::PI;
use std::time;

const WIDTH: usize = 500;
const HEIGHT: usize = 300;
const TILE_SIZE: usize = 10;
const MAP_WIDTH: usize = WIDTH / 25;
const MAP_HEIGHT: usize = HEIGHT / 25;
const MAP_SIZE: usize = MAP_WIDTH * MAP_HEIGHT;
const FOV: f32 = 90.0 / 180.0 * PI;

const PLAYER_SIZE: f32 = 1.0;

mod math;
use math::*;
mod surface;
use surface::*;
mod drawing;
use drawing::*;
mod file;
use file::*;
use std::cmp::Ordering;

struct Player {
    pos: Vec2<f32>,
    size: f32,
    look_angle: f32,
}

struct DepthBufferInfo<'a> {
    distance: f32,
    column: i32,
    info_type: BufferInfoType<'a>,
}

impl Ord for DepthBufferInfo<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.total_cmp(&other.distance)
    }
}
impl PartialOrd for DepthBufferInfo<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
impl PartialEq for DepthBufferInfo<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl Eq for DepthBufferInfo<'_> {}

enum BufferInfoType<'a> {
    Wall,
    Sprite { surf: &'a Surface },
}

fn main() {
    let mut screen = Surface::empty(WIDTH, HEIGHT);
    let image = load_bmp("assets/player.bmp").expect("couldnt load");
    let mut tiles = vec![0u8; WIDTH * HEIGHT];

    let mut player = Player {
        pos: Vec2::new(2.0, 2.0),
        size: PLAYER_SIZE,
        look_angle: 0.0,
    };
    let enemy_pos = Vec2::new(9.5, 9.5);

    tiles[0..MAP_WIDTH].iter_mut().for_each(|val| *val = 1);
    tiles[(MAP_SIZE - MAP_WIDTH)..MAP_SIZE]
        .iter_mut()
        .for_each(|val| *val = 1);
    for i in 0..MAP_HEIGHT {
        tiles[i * MAP_WIDTH] = 1;
        tiles[i * MAP_WIDTH + MAP_WIDTH - 1] = 1;
    }

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

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    let mut depth_buffer: BinaryHeap<DepthBufferInfo> = BinaryHeap::with_capacity(WIDTH + 10);

    let mut now = time::SystemTime::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = now.elapsed().unwrap().as_secs_f32();
        now = time::SystemTime::now();
        screen.fill(0);

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

        let m_pos = Vec2::from_tuple(window.get_mouse_pos(MouseMode::Clamp).unwrap()) / 2.0;

        let m_dir: Vec2<f32> = Vec2::new(0.0, -1.0).rotate(player.look_angle);
        let ray_start = player.pos.clone();
        let rays: Vec<Vec2<f32>> = (0..WIDTH)
            .map(|i| {
                let a = (i as f32 / WIDTH as f32 - 0.5) * FOV;
                m_dir.rotate(a)
            })
            .collect();

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

                if map_check.x >= 0
                    && map_check.x < MAP_WIDTH as i32
                    && map_check.y >= 0
                    && map_check.y < MAP_HEIGHT as i32
                {
                    let index = map_check.x + map_check.y * MAP_WIDTH as i32;
                    if tiles[index as usize] == 1 {
                        tile_found = true;
                    }
                }
            }
            if tile_found {
                let intersection = ray_start + ray_dir * distance;

                let distance = distance * (-FOV / 2.0 + (FOV / WIDTH as f32) * index as f32).cos();

                depth_buffer.push(DepthBufferInfo {
                    distance,
                    column: index as i32,
                    info_type: BufferInfoType::Wall,
                });
                draw_rect(
                    &mut screen,
                    (intersection * TILE_SIZE as f32).as_i32() - Vec2::new(4, 4),
                    Vec2::new(7, 7),
                    0xFFFF,
                );
            }
        }

        let mut vel = Vec2::new(0.0, 0.0);
        window.get_keys().iter().for_each(|key| match key {
            Key::A => vel = vel + Vec2::new(-5.0, 0.0).rotate(player.look_angle) * dt,
            Key::D => vel = vel + Vec2::new(5.0, 0.0).rotate(player.look_angle) * dt,
            Key::W => vel = vel + Vec2::new(0.0, -5.0).rotate(player.look_angle) * dt,
            Key::S => vel = vel + Vec2::new(0.0, 5.0).rotate(player.look_angle) * dt,
            Key::Left => player.look_angle -= 2.0 * dt,
            Key::Right => player.look_angle += 2.0 * dt,
            _ => (),
        });

        let mut new_pos = player.pos.clone();

        new_pos.x += vel.x;
        if vel.x > 0.0 {
            if tiles[(new_pos.x + player.size / 2.0) as usize + new_pos.y as usize * MAP_WIDTH] == 1
            {
                new_pos.x = new_pos.x.ceil() - player.size / 2.0;
            }
        } else {
            if tiles[(new_pos.x - player.size / 2.0) as usize + new_pos.y as usize * MAP_WIDTH] == 1
            {
                new_pos.x = new_pos.x.floor() + player.size / 2.0;
            }
        }

        new_pos.y += vel.y;
        if vel.y > 0.0 {
            if tiles[new_pos.x as usize + (new_pos.y + player.size / 2.0) as usize * MAP_WIDTH] == 1
            {
                new_pos.y = new_pos.y.ceil() - player.size / 2.0;
            }
        } else {
            if tiles[new_pos.x as usize + (new_pos.y - player.size / 2.0) as usize * MAP_WIDTH] == 1
            {
                new_pos.y = new_pos.y.floor() + player.size / 2.0;
            }
        }

        player.pos = new_pos;

        //ENEMY DRAWING
        let camera_plane = Vec2::new(1.0, 0.0).rotate(player.look_angle);
        let camera_normal = Vec2::new(camera_plane.y, -camera_plane.x);

        let enemy_offset_pos = enemy_pos - player.pos;
        let enemy_projected_pos =
            camera_plane * enemy_offset_pos.x + camera_normal * enemy_offset_pos.y;

        let angle = enemy_projected_pos.x.atan2(enemy_projected_pos.y);

        let column = ((angle + FOV) / FOV * WIDTH as f32) as i32 - WIDTH as i32 / 2;
        if enemy_projected_pos.y > 0.0 && angle.abs() < FOV / 2.0 {
            depth_buffer.push(DepthBufferInfo {
                distance: enemy_projected_pos.y,
                column,
                info_type: BufferInfoType::Sprite { surf: &image },
            });
        }

        draw_line(
            &mut screen,
            (player.pos * TILE_SIZE as f32).as_i32(),
            ((player.pos + camera_plane) * TILE_SIZE as f32).as_i32(),
            0xff,
        );
        draw_line(
            &mut screen,
            (player.pos * TILE_SIZE as f32).as_i32(),
            ((player.pos + camera_normal) * TILE_SIZE as f32).as_i32(),
            0xffff,
        );

        for _ in 0..depth_buffer.len() {
            let buf_info = depth_buffer.pop().unwrap();
            let value = (1.0 / buf_info.distance).min(1.0);
            match buf_info.info_type {
                BufferInfoType::Wall => {
                    let height = (value * 1.0 * HEIGHT as f32) as i32;
                    let col = val_from_rgb(0, 0, ((value).sqrt() * 255.0) as u32);
                    draw_rect(
                        &mut screen,
                        Vec2::new(buf_info.column, HEIGHT as i32 / 2 - height / 2),
                        Vec2::new(1, height),
                        col,
                    );
                }
                BufferInfoType::Sprite { surf } => {

                    screen.blit_scaled(surf, Vec2::new(buf_info.column, HEIGHT as i32 / 2), 1.0/buf_info.distance * 20.0);
                }
            }
        }
        depth_buffer.clear();

        screen.blit_scaled(&image, m_pos.as_i32(), 4.0);
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
        screen.blit(
            &image,
            ((player.pos.x - player.size / 2.0) * TILE_SIZE as f32) as i32,
            ((player.pos.y - player.size / 2.0) * TILE_SIZE as f32) as i32,
        );
        window
            .update_with_buffer(&screen.pixel_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
