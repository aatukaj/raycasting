use kira::dsp::Frame;
use minifb::{Key, Window, WindowOptions};
use std::collections::{BinaryHeap, HashMap};

use std::f32::consts::PI;
use std::sync::Arc;
use std::time;

use kira::{
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use simple_logger::SimpleLogger;

mod math;
use math::*;
mod surface;
use surface::*;
mod drawing;
use drawing::*;
mod file;
use file::*;

mod depth_buffer;
use depth_buffer::*;

mod entity;
use entity::*;

mod rect;

mod tile_map;
use tile_map::*;

const WIDTH: usize = 500;
const HEIGHT: usize = 300;

const FOV: f32 = 90.0 / 180.0 * PI;

const PLAYER_SIZE: f32 = 0.8;

//bit flags
const PLAYER: u32 = 0b1;
const ENEMY: u32 = 0b1 << 1;
const PROJECTILE: u32 = 0b1 << 2;

pub struct AssetCache {
    sprites: HashMap<String, Surface>,
    sounds: HashMap<String, StaticSoundData>,
}
impl AssetCache {
    fn new() -> Self {
        AssetCache {
            sprites: HashMap::new(),
            sounds: HashMap::new(),
        }
    }
    pub fn load_bmp(&mut self, path: &str) -> &Surface {
        self.sprites
            .entry(path.to_string())
            .or_insert_with(|| match load_bmp(path) {
                Ok(img) => img,
                Err(err) => {
                    log::warn!("Couldn't load {path}, ERROR: {err}");
                    let mut surf = Surface::empty(16, 16);
                    surf.fill(0xDA70D6);
                    surf
                }
            })
    }
    pub fn load_sound(&mut self, path: &str) -> &StaticSoundData {
        self.sounds.entry(path.to_string()).or_insert_with(|| {
            StaticSoundData::from_file(path, StaticSoundSettings::default()).unwrap_or_else(|err| {
                log::warn!("Couldn't load {path}, ERROR: {err}");
                StaticSoundData {
                    //if the sound file doesnt exits, return a dummy sound
                    sample_rate: 0,
                    frames: Arc::new([Frame::new(0.0, 0.0)]),
                    settings: StaticSoundSettings::default(),
                }
            })
        })
    }
}

pub trait Component {
    fn update(&self, entity: &mut Entity, game: &mut Game, dt: f32);
}

pub struct BasicCollisionComponent;
impl Component for BasicCollisionComponent {
    fn update(&self, entity: &mut Entity, game: &mut Game, dt: f32) {
        entity.rect.pos = entity.pos;
        entity.rect.pos.x += entity.vel.x * dt;
        let h_size = entity.rect.size / 2.0;
        let cols = game.tile_map.get_collisions(&entity.rect);
        for col in cols {
            if entity.vel.x > 0.0 {
                entity.rect.pos.x = col.x as f32 - h_size - 0.00420; //i dont like the arbitrary subtraction but it fixes a bug
            } else {
                entity.rect.pos.x = col.x as f32 + 1.0 + h_size;
            }
        }
        for other in game.entities.iter() {
            if other.collidable && entity.rect.collide(&other.rect) {
                if entity.vel.x > 0.0 {
                    entity.rect.set_right(other.rect.get_left());
                } else {
                    entity.rect.set_left(other.rect.get_right());
                }
            }
        }

        entity.rect.pos.y += entity.vel.y * dt;
        let cols = game.tile_map.get_collisions(&entity.rect);
        for col in cols {
            if entity.vel.y > 0.0 {
                entity.rect.pos.y = col.y as f32 - h_size - 0.00420;
            } else {
                entity.rect.pos.y = col.y as f32 + 1.0 + h_size;
            }
        }
        for other in game.entities.iter() {
            if other.collidable && entity.rect.collide(&other.rect) {
                if entity.vel.y > 0.0 {
                    entity.rect.set_bottom(other.rect.get_top());
                } else {
                    entity.rect.set_top(other.rect.get_bottom());
                }
            }
        }
        entity.pos = entity.rect.pos;
    }
}
pub struct ProjectileCollisionComponent;
impl Component for ProjectileCollisionComponent {
    fn update(&self, entity: &mut Entity, game: &mut Game, dt: f32) {
        entity.pos = entity.pos + entity.vel * dt;
        entity.rect.pos = entity.pos;
        let explosion_sound = game
            .assets
            .load_sound("assets/sounds/explosionCrunch_000.ogg")
            .clone();
        for other in &game.entities {
            if other.rect.collide(&entity.rect) {
                entity.alive = false;
                game.audio_manager.play(explosion_sound).unwrap();
                return;
            }
        }
        if !game.tile_map.get_collisions(&entity.rect).is_empty() {
            entity.alive = false;
            game.audio_manager.play(explosion_sound).unwrap();
        }
    }
}

pub struct PlayerInputComponent;
impl Component for PlayerInputComponent {
    fn update(&self, entity: &mut Entity, game: &mut Game, dt: f32) {
        let mut vel = Vec2::new(0.0, 0.0);
        let player = entity;
        game.window.get_keys().iter().for_each(|key| match key {
            Key::A => vel = vel + Vec2::new(-5.0, 0.0).rotate(player.look_angle),
            Key::D => vel = vel + Vec2::new(5.0, 0.0).rotate(player.look_angle),
            Key::W => vel = vel + Vec2::new(0.0, -5.0).rotate(player.look_angle),
            Key::S => vel = vel + Vec2::new(0.0, 5.0).rotate(player.look_angle),
            Key::Left => player.look_angle -= 2.0 * dt,
            Key::Right => player.look_angle += 2.0 * dt,

            _ => (),
        });

        player.vel = vel;
        game.window
            .get_keys_pressed(minifb::KeyRepeat::No)
            .iter()
            .for_each(|key| match key {
                Key::Space => {
                    let dir = Vec2::new(0.0, -1.0).rotate(player.look_angle);
                    game.entities.push(Entity::new(
                        player.pos + dir * 0.5,
                        Some("assets/bullet.bmp"),
                        dir * 8.0,
                        0.3,
                        false,
                        vec![Box::new(ProjectileCollisionComponent)],
                    ));
                    let sound_data = game.assets.load_sound("assets/sounds/laserRetro_002.ogg");
                    game.audio_manager.play(sound_data.clone()).unwrap();
                }
                _ => (),
            })
    }
}

pub struct Game<'a> {
    pub window: Window,
    pub depth_buffer: DepthBufferRenderer<'a>,
    pub tile_map: TileMap,
    pub entities: Vec<Entity<'a>>,
    pub screen: Surface,
    pub assets: AssetCache,
    pub audio_manager: AudioManager,
}
impl Game<'_> {
    fn new() -> Self {
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
            assets: AssetCache::new(),
            audio_manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .unwrap(),
        }
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let mut game = Game::new();

    game.entities = vec![
        Entity::new(
            Vec2::new(2.5, 2.5),
            Some("assets/player.bmp"),
            Vec2::new(0.0, 0.0),
            PLAYER_SIZE,
            true,
            vec![
                Box::new(BasicCollisionComponent),
                Box::new(PlayerInputComponent),
            ],
        ),
        Entity::new(
            Vec2::new(9.5, 9.5),
            Some("assets/player.bmp"),
            Vec2::new(0.0, 0.0),
            0.5,
            true,
            vec![Box::new(BasicCollisionComponent)],
        ),
        Entity::new(
            Vec2::new(12.0, 8.5),
            Some("assets/bullet.bmp"),
            Vec2::new(0.0, 0.0),
            1.5,
            true,
            vec![Box::new(BasicCollisionComponent)],
        ),
    ];
    game.entities.swap(0, 1);
    // Limit to max ~60 fps update rate
    game.window
        .limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut now = time::SystemTime::now();
    while game.window.is_open() && !game.window.is_key_down(Key::Escape) {
        let dt = now.elapsed().unwrap().as_secs_f32();

        now = time::SystemTime::now();
        game.screen.fill(0);

        render_bg(&mut game.screen);

        //let m_pos = Vec2::from_tuple(window.get_mouse_pos(MouseMode::Clamp).unwrap()) / 2.0;

        for i in (0..=0).chain(0..(game.entities.len() - 1)) {
            let mut entity = game.entities.swap_remove(i.min(game.entities.len() - 1));
            entity.update(dt, &mut game);
            if entity.alive {
                game.entities.push(entity);
            }
        }

        cast_rays(
            &game.tile_map,
            &mut game.depth_buffer.data,
            &mut game.screen,
            &game.entities[0],
        );
        project_entities(&mut game);

        game.depth_buffer.render(&mut game.screen, &mut game.assets);

        let surf_to_blit = game.assets.load_bmp("assets/gun.bmp");
        game.screen.blit_scaled(
            surf_to_blit,
            Vec2::new(
                (WIDTH / 2) as i32,
                (HEIGHT - surf_to_blit.width / 2 - 14) as i32,
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

fn render_bg(screen: &mut Surface) {
    for y in 0..(HEIGHT / 2) {
        let brigthness = 1.0 - (y as f32 / (HEIGHT / 2) as f32).sqrt();
        let value = brigthness;

        draw_rect(
            screen,
            Vec2::new(0, y as i32),
            Vec2::new(WIDTH as i32, 1),
            set_value_brightness(0x516988, value),
        );
        draw_rect(
            screen,
            Vec2::new(0, (HEIGHT - y) as i32),
            Vec2::new(WIDTH as i32, 1),
            set_value_brightness(0xc0cbdc, value),
        );
    }
}

fn project_entities<'a>(game: &mut Game<'a>) {
    let player = &game.entities[0];
    let camera_plane = Vec2::new(1.0, 0.0).rotate(player.look_angle);
    let camera_normal = Vec2::new(camera_plane.y, -camera_plane.x);

    for entity in game.entities.iter() {
        if let Some(sprite) = entity.sprite {
            let enemy_offset_pos = entity.pos - player.pos;
            let enemy_projected_pos =
                camera_plane * enemy_offset_pos.x + camera_normal * enemy_offset_pos.y;

            let angle = enemy_projected_pos.x.atan2(enemy_projected_pos.y);

            let column = ((angle + FOV) / FOV * WIDTH as f32) as i32 - WIDTH as i32 / 2;
            if enemy_projected_pos.y > -0.1 && angle.abs() < FOV / 1.5 {
                game.depth_buffer.data.push(DepthBufferData {
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

            let percentage = match direction {
                Direction::Horizontal => intersection.x.fract(),
                Direction::Vertical => intersection.y.fract(),
            };

            depth_buffer.push(DepthBufferData {
                distance,
                column: index as i32,
                data_type: BufferDataType::Wall {
                    direction,
                    percentage,
                },
            });
        }
    }
}

fn handle_input(dt: f32, game: &mut Game) {
    let mut vel = Vec2::new(0.0, 0.0);
    let player = &mut game.entities[0];
    game.window.get_keys().iter().for_each(|key| match key {
        Key::A => vel = vel + Vec2::new(-5.0, 0.0).rotate(player.look_angle),
        Key::D => vel = vel + Vec2::new(5.0, 0.0).rotate(player.look_angle),
        Key::W => vel = vel + Vec2::new(0.0, -5.0).rotate(player.look_angle),
        Key::S => vel = vel + Vec2::new(0.0, 5.0).rotate(player.look_angle),
        Key::Left => player.look_angle -= 2.0 * dt,
        Key::Right => player.look_angle += 2.0 * dt,

        _ => (),
    });

    game.entities[0].vel = vel;
    game.window
        .get_keys_pressed(minifb::KeyRepeat::No)
        .iter()
        .for_each(|key| match key {
            Key::Space => {
                let dir = Vec2::new(0.0, -1.0).rotate(game.entities[0].look_angle);
                game.entities.push(Entity::new(
                    game.entities[0].pos + dir * 0.5,
                    Some("assets/bullet.bmp"),
                    dir * 8.0,
                    0.3,
                    false,
                    vec![Box::new(ProjectileCollisionComponent)],
                ));
                let sound_data = game.assets.load_sound("assets/sounds/laserRetro_002.ogg");
                game.audio_manager.play(sound_data.clone()).unwrap();
            }
            _ => (),
        })
}
