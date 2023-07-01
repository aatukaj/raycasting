use minifb::{Key, Window, WindowOptions};
use std::collections::{BinaryHeap, HashMap};

use std::error::Error;
use std::f32::consts::PI;
use std::time;

mod math;
use math::*;
mod surface;
use surface::*;
mod drawing;
use drawing::*;
mod file;
use file::*;

use std::fs;

mod depth_buffer;
use depth_buffer::*;

mod rect;
use rect::*;

const WIDTH: usize = 500;
const HEIGHT: usize = 300;

const FOV: f32 = 90.0 / 180.0 * PI;

const PLAYER_SIZE: f32 = 0.8;

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

struct Game<'a> {
    window: Window,
    depth_buffer: DepthBufferRenderer<'a>,
    tile_map: TileMap,
    entities: Vec<Entity<'a>>,
    screen: Surface,
    sprites: HashMap<String, Surface>,
}
impl<'a> Game<'a> {
    fn new() -> Self {
        let mut sprites = HashMap::new();
        sprites.insert(
            "player".to_string(),
            load_bmp("assets/player.bmp").expect("couldnt load"),
        );
        Game {
            window: Window::new(
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
            }),
            depth_buffer: DepthBufferRenderer::new(WIDTH + 10),
            entities: Vec::new(),
            tile_map: load_map("assets/map.txt").expect("couldn't load map"),
            screen: Surface::empty(WIDTH, HEIGHT),
            sprites,
        }
    }
}

fn main() {
    let mut game = Game::new();

    let image = load_bmp("assets/player.bmp").expect("couldnt load");
    let gun_image = load_bmp("assets/gun.bmp").unwrap();
    let bullet_image = load_bmp("assets/bullet.bmp").unwrap();

    game.entities = vec![
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

    // Limit to max ~60 fps update rate
    game.window
        .limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut now = time::SystemTime::now();
    while game.window.is_open() && !game.window.is_key_down(Key::Escape) {
        let dt = now.elapsed().unwrap().as_secs_f32();

        now = time::SystemTime::now();
        game.screen.fill(0);

        for y in 0..(HEIGHT / 2) {
            let brigthness = 1.0 - (y as f32 / (HEIGHT / 2) as f32).sqrt();
            let value = brigthness;

            draw_rect(
                &mut game.screen,
                Vec2::new(0, y as i32),
                Vec2::new(WIDTH as i32, 1),
                val_from_rgb(
                    (0x5a as f32 * value) as u32,
                    (0x69 as f32 * value) as u32,
                    (0x88 as f32 * value) as u32,
                ),
            );
            draw_rect(
                &mut game.screen,
                Vec2::new(0, (HEIGHT - y) as i32),
                Vec2::new(WIDTH as i32, 1),
                val_from_rgb(
                    (0xc0 as f32 * value) as u32,
                    (0xcb as f32 * value) as u32,
                    (0xdc as f32 * value) as u32,
                ),
            );
        }

        //let m_pos = Vec2::from_tuple(window.get_mouse_pos(MouseMode::Clamp).unwrap()) / 2.0;
        handle_input(&game.window, dt, &mut game.entities, &bullet_image);

        let mut entity = game.entities.swap_remove(0);
        entity.update(dt, &game.tile_map, &game.entities);
        game.entities.push(entity);
        for i in 0..(game.entities.len() - 1) {
            let mut entity = game.entities.swap_remove(i);
            entity.update(dt, &game.tile_map, &game.entities);
            game.entities.push(entity);
        }

        cast_rays(
            &game.tile_map,
            &mut game.depth_buffer.data,
            &mut game.screen,
            &game.entities[0],
        );
        project_entities(&game.entities, &mut game.depth_buffer.data);

        game.depth_buffer.render(&mut game.screen);

        game.screen.blit_scaled(
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
        game.window
            .update_with_buffer(&game.screen.pixel_buffer, WIDTH, HEIGHT)
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

fn cast_rays(
    tile_map: &TileMap,
    depth_buffer: &mut BinaryHeap<DepthBufferData<'_>>,
    screen: &mut Surface,
    player: &Entity,
) {
    let m_dir: Vec2<f32> = Vec2::new(0.0, -1.0).rotate(player.look_angle);
    let ray_start = player.pos;
    let rays: Vec<Vec2<f32>> = (0..screen.width)
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

            let distance =
                distance * (-FOV / 2.0 + (FOV / screen.width as f32) * index as f32).cos();

            depth_buffer.push(DepthBufferData {
                distance,
                column: index as i32,
                data_type: BufferDataType::Wall(direction),
            });
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
