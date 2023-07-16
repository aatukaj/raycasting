use kira::dsp::Frame;
use minifb::{Key, Window, WindowOptions};
use std::cell::Ref;
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use std::sync::Arc;
use std::time;

use glam::*;
use kira::{
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
};
use simple_logger::SimpleLogger;

mod math;
use math::set_value_brightness;

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

mod components;
use components::*;

const WIDTH: usize = 700 * 2;
const HEIGHT: usize = 400 * 2;
const SCALE: usize = 1;
const PLAYER_SIZE: f32 = 0.8;

pub struct AssetCache {
    sprites: RefCell<HashMap<String, Arc<Surface>>>,
    sounds: HashMap<String, StaticSoundData>,
}
impl AssetCache {
    fn new() -> Self {
        AssetCache {
            sprites: RefCell::new(HashMap::new()),
            sounds: HashMap::new(),
        }
    }
    pub fn load_png(&self, path: &str) -> Arc<Surface> {
        self.sprites
            .borrow_mut()
            .entry(path.to_string())
            .or_insert_with(|| match load_png(path) {
                Ok(img) => Arc::new(img),
                Err(err) => {
                    log::warn!("Couldn't load {path}, ERROR: {err}");
                    let mut surf = Surface::empty(16, 16);
                    surf.fill(0xDA70D6);
                    Arc::new(surf)
                }
            })
            .clone()
    }

    pub fn load_sound(
        &mut self,
        path: &str,
        settings: Option<StaticSoundSettings>,
    ) -> StaticSoundData {
        self.sounds
            .entry(path.to_string())
            .or_insert_with(|| {
                StaticSoundData::from_file(path, settings.unwrap_or_default()).unwrap_or_else(
                    |err| {
                        log::warn!("Couldn't load {path}, ERROR: {err}");
                        StaticSoundData {
                            //if the sound file doesnt exits, return a dummy sound
                            sample_rate: 0,
                            frames: Arc::new([Frame::new(0.0, 0.0)]),
                            settings: StaticSoundSettings::default(),
                        }
                    },
                )
            })
            .clone()
    }
}

pub struct Game<'a> {
    pub window: Window,
    pub renderer: DepthBufferRenderer<'a>,
    pub tile_map: TileMap<'a>,

    pub screen: Surface,
    pub assets: AssetCache,
    pub audio_manager: AudioManager,
    pub entities: HashMap<u32, Entity<'a>>,
    next_id: u32,
}
impl<'a> Game<'a> {
    fn new() -> Self {
        Game {
            window: Window::new(
                "Test - ESC to exit",
                WIDTH * SCALE,
                HEIGHT * SCALE,
                WindowOptions {
                    scale_mode: minifb::ScaleMode::AspectRatioStretch,
                    ..Default::default()
                },
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            }),
            renderer: DepthBufferRenderer::new(WIDTH + 10),
            entities: HashMap::new(),
            tile_map: load_map("assets/map.txt").expect("couldn't load map"),
            screen: Surface::empty(WIDTH, HEIGHT),
            assets: AssetCache::new(),
            audio_manager: AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                .unwrap(),
            next_id: 0,
        }
    }
    fn add_entity(&mut self, mut entity: Entity<'a>) {
        entity.id = self.next_id;
        self.entities.insert(self.next_id, entity);
        self.next_id += 1;
    }
}

fn main() {
    SimpleLogger::new()
        .with_colors(true)
        .with_level(log::LevelFilter::Off)
        .with_module_level("raycasting", log::LevelFilter::Trace)
        .init()
        .unwrap();

    let mut game = Game::new();
    let gun_image = load_png("assets/gun.png").unwrap();
    let music = game.assets.load_sound(
        "assets/sounds/game_bg.mp3",
        Some(
            StaticSoundSettings::default()
                .volume(0.2)
                .loop_region(0.0..),
        ),
    );
    game.audio_manager.play(music.clone());
    game.add_entity(Entity::new(
        Vec2::new(6.5, 7.5),
        Some("assets/player.png"),
        Vec2::new(0.0, 0.0),
        PLAYER_SIZE,
        true,
        vec![
            Box::new(BasicCollisionComponent),
            Box::new(PlayerInputComponent),
            Box::new(CameraComponent::new()),
        ],
    ));

    game.add_entity(Entity::new(
        Vec2::new(9.5, 9.5),
        Some("assets/player.png"),
        Vec2::new(0.0, 0.0),
        0.6,
        true,
        vec![
            Box::new(BasicCollisionComponent),
            Box::new(BasicAiComponent),
            Box::new(DeathComponent::new("assets/sounds/death.wav")),
        ],
    ));
    game.add_entity(Entity::new(
        Vec2::new(10.5, 9.5),
        Some("assets/guy.png"),
        Vec2::new(0.0, 0.0),
        0.6,
        true,
        vec![
            Box::new(BasicCollisionComponent),
            Box::new(BasicAiComponent),
            Box::new(DeathComponent::new("assets/sounds/death.wav")),
        ],
    ));
    game.add_entity(Entity::new(
        Vec2::new(12.5, 8.5),
        Some("assets/key.png"),
        Vec2::new(0.0, 0.0),
        0.2,
        true,
        vec![Box::new(BasicCollisionComponent)],
    ));

    // Limit to max ~60 fps update rate
    game.window
        .limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    game.tile_map
        .tile_update_indeces
        .push(6 + 6 * game.tile_map.width);
    let mut now = time::SystemTime::now();
    while game.window.is_open() && !game.window.is_key_down(Key::Escape) {
        let dt = now.elapsed().unwrap().as_secs_f32();

        now = time::SystemTime::now();
        game.screen.fill(0);

        let keys = game.entities.keys().copied().collect::<Vec<_>>();

        for key in keys {
            let mut entity = game.entities.remove(&key).unwrap();
            entity.update(dt, &mut game);
            if entity.alive {
                game.entities.insert(key, entity);
            }
        }
        game.tile_map.update(dt);
        game.renderer.render(&mut game.screen, &mut game.assets);

        let surf_to_blit = &gun_image;
        game.screen.blit_scaled(
            surf_to_blit,
            IVec2::new(
                (WIDTH / 2) as i32,
                (HEIGHT - surf_to_blit.width / 2 - 70) as i32,
            ),
            6.0,
        );

        game.window
            .update_with_buffer(&game.screen.pixel_buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

