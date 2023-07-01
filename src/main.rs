use minifb::{Key, Window, WindowOptions};
use std::collections::{BinaryHeap};

use std::error::Error;
use std::f32::consts::PI;
use std::time;

use rand::prelude::*;

mod math;
use math::*;
mod surface;
use surface::*;
mod drawing;
use drawing::*;
mod file;
use file::*;
use std::cmp::Ordering;
use std::fs;

const WIDTH: usize = 500;
const HEIGHT: usize = 300;
const TILE_SIZE: usize = 10;

const FOV: f32 = 90.0 / 180.0 * PI;

const PLAYER_SIZE: f32 = 0.8;

enum Direction {
    Horizontal,
    Vertical,
}

struct TileMap {
    width: usize,
    height: usize,
    buf: Vec<u8>,
}
impl TileMap {
    fn get_tile(&self, pos: Vec2<i32>) -> u8 {
        if 0 <= pos.x && pos.x < self.width as i32 && 0 <= pos.y && pos.y < self.height as i32 {
            return self.buf[pos.x as usize + pos.y as usize * self.width];
        }
        0
    }
    fn get_collisions<'a>(&'a self, rect: &'a Rect) -> Vec<Vec2<i32>> {
        rect.get_corners()
            .iter()
            .map(|pos| pos.as_i32())
            .filter(|pos| self.get_tile(*pos) != 0)
            .collect()
    }
}

#[derive(PartialEq, Debug)]
struct Rect {
    pos: Vec2<f32>,
    size: f32,
}
impl Rect {
    fn get_corners(&self) -> [Vec2<f32>; 4] {
        let h_size = self.size / 2.0;
        [
            self.pos + Vec2::new(h_size, h_size),
            self.pos + Vec2::new(-h_size, h_size),
            self.pos + Vec2::new(-h_size, -h_size),
            self.pos + Vec2::new(h_size, -h_size),
        ]
    }

    fn get_top(&self) -> f32 {
        self.pos.y - self.size / 2.0
    }
    fn get_bottom(&self) -> f32 {
        self.pos.y + self.size / 2.0
    }
    fn get_left(&self) -> f32 {
        self.pos.x - self.size / 2.0
    }
    fn get_right(&self) -> f32 {
        self.pos.x + self.size / 2.0
    }
    fn set_top(&mut self, y: f32) {
        self.pos.y = y + self.size / 2.0;
    }
    fn set_bottom(&mut self, y: f32) {
        self.pos.y = y - self.size / 2.0;
    }
    fn set_left(&mut self, x: f32) {
        self.pos.x = x + self.size / 2.0;
    }
    fn set_right(&mut self, x: f32) {
        self.pos.x = x - self.size / 2.0;
    }
    fn collide(&self, other: &Rect) -> bool {
        let a_min = Vec2::new(self.get_left(), self.get_top());
        let a_max = Vec2::new(self.get_right(), self.get_bottom());
        let b_min = Vec2::new(other.get_left(), other.get_top());
        let b_max = Vec2::new(other.get_bottom(), other.get_right());

        a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y
    }
}

fn load_map(path: &str) -> Result<TileMap, Box<dyn Error>> {
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

struct Entity<'a> {
    pos: Vec2<f32>,
    sprite: Option<&'a Surface>,
    look_angle: f32,
    vel: Vec2<f32>,

    rect: Rect,
    collidable: bool,
}
impl<'a> Entity<'a> {
    fn new(
        pos: Vec2<f32>,
        sprite: Option<&'a Surface>,
        vel: Vec2<f32>,
        size: f32,
        collidable: bool,
    ) -> Self {
        Entity {
            pos,
            sprite,
            look_angle: 0.0,
            vel,
            rect: Rect { pos, size },
            collidable,
        }
    }

    fn update(&mut self, dt: f32, tile_map: &TileMap, entities: &[Entity]) {
        self.rect.pos = self.pos;
        self.rect.pos.x += self.vel.x * dt;
        let h_size = self.rect.size / 2.0;
        let cols = tile_map.get_collisions(&self.rect);
        for col in cols {
            if self.vel.x > 0.0 {
                self.rect.pos.x = col.x as f32 - h_size - 0.00420; //i dont like the arbitrary subtraction but it fixes a bug
            } else {
                self.rect.pos.x = col.x as f32 + 1.0 + h_size;
            }
        }
        for entity in entities.iter() {
            if entity.collidable && self.rect.collide(&entity.rect) {
                if self.vel.x > 0.0 {
                    self.rect.set_right(entity.rect.get_left());
                } else {
                    self.rect.set_left(entity.rect.get_right());
                }
            }
        }

        self.rect.pos.y += self.vel.y * dt;
        let cols = tile_map.get_collisions(&self.rect);
        for col in cols {
            if self.vel.y > 0.0 {
                self.rect.pos.y = col.y as f32 - h_size - 0.00420;
            } else {
                self.rect.pos.y = col.y as f32 + 1.0 + h_size;
            }
        }
        for entity in entities.iter() {
            if entity.collidable && self.rect.collide(&entity.rect) {
                if self.vel.y > 0.0 {
                    self.rect.set_bottom(entity.rect.get_top());
                } else {
                    self.rect.set_top(entity.rect.get_bottom());
                }
            }
        }

        self.pos = self.rect.pos;
    }
}

struct DepthBufferData<'a> {
    distance: f32,
    column: i32,
    data_type: BufferDataType<'a>,
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

enum BufferDataType<'a> {
    Wall(Direction),
    Sprite { surf: &'a Surface },
}

fn main() {
    let tile_map = load_map("assets/map.txt").unwrap();
    let mut screen = Surface::empty(WIDTH, HEIGHT);
    let image = load_bmp("assets/player.bmp").expect("couldnt load");
    let gun_image = load_bmp("assets/gun.bmp").unwrap();
    let bullet_image = load_bmp("assets/bullet.bmp").unwrap();

    let mut entities = vec![
        Entity::new(
            Vec2::new(2.5, 2.5),
            None,
            Vec2::new(0.0, 0.0),
            PLAYER_SIZE,
            true,
        ),
        Entity::new(
            Vec2::new(9.5, 9.5),
            Some(&image),
            Vec2::new(0.0, 0.0),
            0.5,
            true,
        ),
        Entity::new(
            Vec2::new(12.0, 8.5),
            Some(&bullet_image),
            Vec2::new(0.0, 0.0),
            1.5,
            true,
        ),
    ];

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
    let mut depth_buffer: BinaryHeap<DepthBufferData> = BinaryHeap::with_capacity(WIDTH + 10);

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

        //let m_pos = Vec2::from_tuple(window.get_mouse_pos(MouseMode::Clamp).unwrap()) / 2.0;
        handle_input(&window, dt, &mut entities, &bullet_image);

        let mut entity = entities.swap_remove(0);
        entity.update(dt, &tile_map, &entities);
        entities.push(entity);
        for i in 0..(entities.len() - 1) {
            let mut entity = entities.swap_remove(i);
            entity.update(dt, &tile_map, &entities);
            entities.push(entity);
        }

        cast_rays(&tile_map, &mut depth_buffer, &mut screen, &entities[0]);
        project_entities(&entities, &mut depth_buffer);

        /*
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
        */

        render_depth_buffer(&mut depth_buffer, &mut screen);

        screen.blit_scaled(
            &gun_image,
            Vec2::new(
                (WIDTH / 2) as i32,
                (HEIGHT - gun_image.width / 2 - 5) as i32,
            ),
            2.0,
        );
        /*
        for (i, &val) in tile_map.buf.iter().enumerate() {
            if val == 1 {
                draw_rect(
                    &mut screen,
                    Vec2::new(
                        (i % tile_map.width * TILE_SIZE) as i32,
                        (i / tile_map.width * TILE_SIZE) as i32,
                    ),
                    Vec2::new(TILE_SIZE as i32, TILE_SIZE as i32),
                    0xFF,
                )
            }
        }
        for x in 0..tile_map.width {
            draw_rect(
                &mut screen,
                Vec2::new((x * TILE_SIZE) as i32, 0),
                Vec2::new(1, (TILE_SIZE * tile_map.height) as i32),
                0xffffff,
            );
        }
        for y in 0..tile_map.height {
            draw_rect(
                &mut screen,
                Vec2::new(0, (y * TILE_SIZE) as i32),
                Vec2::new((TILE_SIZE * tile_map.width) as i32, 1),
                0xffffff,
            );
        }
        let player = &entities[0];
        screen.blit(
            &image,
            ((player.pos.x - player.size / 2.0) * TILE_SIZE as f32) as i32,
            ((player.pos.y - player.size / 2.0) * TILE_SIZE as f32) as i32,
        );
        */
        window
            .update_with_buffer(&screen.pixel_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

fn project_entities<'a>(
    entities: &Vec<Entity<'a>>,
    depth_buffer: &mut BinaryHeap<DepthBufferData<'a>>,
) {
    let player = &entities[0];
    let camera_plane = Vec2::new(1.0, 0.0).rotate(player.look_angle);
    let camera_normal = Vec2::new(camera_plane.y, -camera_plane.x);

    for entity in entities.iter() {
        if let Some(sprite) = entity.sprite {
            let enemy_offset_pos = entity.pos - player.pos;
            let enemy_projected_pos =
                camera_plane * enemy_offset_pos.x + camera_normal * enemy_offset_pos.y;

            let angle = enemy_projected_pos.x.atan2(enemy_projected_pos.y);

            let column = ((angle + FOV) / FOV * WIDTH as f32) as i32 - WIDTH as i32 / 2;
            if enemy_projected_pos.y > -0.1 && angle.abs() < FOV / 1.5 {
                depth_buffer.push(DepthBufferData {
                    distance: enemy_projected_pos.y,
                    column,
                    data_type: BufferDataType::Sprite { surf: sprite },
                });
            }
        }
    }
}

fn render_depth_buffer(depth_buffer: &mut BinaryHeap<DepthBufferData<'_>>, screen: &mut Surface) {
    for _ in 0..depth_buffer.len() {
        let buf_data: DepthBufferData<'_> = depth_buffer.pop().unwrap();
        let value = (1.0 / buf_data.distance).min(1.0);
        match buf_data.data_type {
            BufferDataType::Wall(direction) => {
                let height = (value * 1.5 * HEIGHT as f32) as i32;
                let value = ((value).sqrt() * 255.0) as u32;
                let col = match direction {
                    Direction::Horizontal => val_from_rgb(0, 0, value),
                    Direction::Vertical => val_from_rgb(0, value, 0),
                };
                draw_rect(
                    screen,
                    Vec2::new(buf_data.column, HEIGHT as i32 / 2 - height / 2),
                    Vec2::new(1, height),
                    col,
                );
            }
            BufferDataType::Sprite { surf } => {
                screen.blit_scaled(
                    surf,
                    Vec2::new(buf_data.column, HEIGHT as i32 / 2),
                    1.0 / buf_data.distance * 20.0,
                );
            }
        }
    }
    depth_buffer.clear();
}

fn cast_rays(
    tile_map: &TileMap,
    depth_buffer: &mut BinaryHeap<DepthBufferData<'_>>,
    screen: &mut Surface,
    player: &Entity,
) {
    let m_dir: Vec2<f32> = Vec2::new(0.0, -1.0).rotate(player.look_angle);
    let ray_start = player.pos;
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
        let mut direction = Direction::Horizontal;
        let max_distance = 100.0;
        let mut distance = 0.0;

        while !tile_found && distance < max_distance {
            if ray_length_1d.x < ray_length_1d.y {
                map_check.x += step.x;
                distance = ray_length_1d.x;
                ray_length_1d.x += ray_unit_step.x;
                direction = Direction::Vertical;
            } else {
                map_check.y += step.y;
                distance = ray_length_1d.y;
                ray_length_1d.y += ray_unit_step.y;
                direction = Direction::Horizontal;
            }

            if tile_map.get_tile(map_check) == 1 {
                tile_found = true;
            }
        }
        if tile_found {
            let intersection = ray_start + ray_dir * distance;

            let distance = distance * (-FOV / 2.0 + (FOV / WIDTH as f32) * index as f32).cos();

            depth_buffer.push(DepthBufferData {
                distance,
                column: index as i32,
                data_type: BufferDataType::Wall(direction),
            });
            /*
            draw_rect(
                screen,
                (intersection * TILE_SIZE as f32).as_i32() - Vec2::new(4, 4),
                Vec2::new(7, 7),
                0xFFFF,
            );
            */
        }
    }
}

fn handle_input<'a>(window: &Window, dt: f32, entities: &mut Vec<Entity<'a>>, bimage: &'a Surface) {
    let mut vel = Vec2::new(0.0, 0.0);
    let player = &mut entities[0];
    window.get_keys().iter().for_each(|key| match key {
        Key::A => vel = vel + Vec2::new(-5.0, 0.0).rotate(player.look_angle),
        Key::D => vel = vel + Vec2::new(5.0, 0.0).rotate(player.look_angle),
        Key::W => vel = vel + Vec2::new(0.0, -5.0).rotate(player.look_angle),
        Key::S => vel = vel + Vec2::new(0.0, 5.0).rotate(player.look_angle),
        Key::Left => player.look_angle -= 2.0 * dt,
        Key::Right => player.look_angle += 2.0 * dt,

        _ => (),
    });

    entities[0].vel = vel;
    window
        .get_keys_pressed(minifb::KeyRepeat::No)
        .iter()
        .for_each(|key| match key {
            Key::Space => {
                let dir = Vec2::new(0.0, -1.0).rotate(entities[0].look_angle);
                entities.push(Entity::new(
                    entities[0].pos + dir * 0.5,
                    Some(bimage),
                    dir * 8.0,
                    0.3,
                    false,
                ))
            }
            _ => (),
        })
}
