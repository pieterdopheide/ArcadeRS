#[macro_use]
mod events;
pub mod data;
pub mod gfx;

use self::gfx::Sprite;
use sdl2::render::Renderer;
use sdl2::pixels::Color;
use std::collections::HashMap;
use std::path::Path;

struct_events! {
    keyboard: {
        key_escape: Escape,
        key_up: Up,
        key_down: Down,
        key_left: Left,
        key_right: Right,
        key_space: Space,
        key_enter: Return,

        key_1: Num1,
        key_2: Num2,
        key_3: Num3
    },
    else: {
        quit: Quit { .. }
    }
}

pub struct Phi<'window> {
    pub events: Events,
    pub renderer: Renderer<'window>,

    allocated_channels: isize,
    cached_fonts: HashMap<(&'static str, i32), ::sdl2_ttf::Font>,
}

impl<'window> Phi<'window> {
    fn new(events: Events, renderer: Renderer<'window>) -> Phi<'window> {
        let allocated_channels = 32;
        ::sdl2_mixer::allocate_channels(allocated_channels);

        Phi {
            events: events,
            renderer: renderer,
            allocated_channels: allocated_channels,
            cached_fonts: HashMap::new(),
        }
    }

    pub fn output_size(&self) -> (f64, f64) {
        let (w, h) = self.renderer.output_size().unwrap();
        (w as f64, h as f64)
    }

    // Renders a string of text as a sprite using the provided parameters
    pub fn ttf_str_sprite(&mut self, text: &str, font_path: &'static str, size: i32, color: Color) -> Option<Sprite> {
        let ttf_context = ::sdl2_ttf::init().unwrap();
        let mut font = ttf_context.load_font(Path::new(font_path), size as u16).unwrap();
        font.render(text).blended(color).ok()
            .and_then(|surface| self.renderer.create_texture_from_surface(&surface).ok())
            .map(Sprite::new)
    }

    // Play a sound once, and allocate new channels if this is necessary
    pub fn play_sound(&mut self, sound: &::sdl2_mixer::Chunk) {
        // Attempt to play the sound once
        match ::sdl2_mixer::Channel::all().play(sound, 0) {
            Err(_) => {
                // If there weren't enough channels allocated, then double the number and try again
                self.allocated_channels *= 2;
                ::sdl2_mixer::allocate_channels(self.allocated_channels);
                self.play_sound(sound);
            },

            _ => { /* Everything's alright! */ }
        }
    }
}

pub enum ViewAction {
    Render(Box<View>),
    Quit,
}

pub trait View {
    fn update(self: Box<Self>, context: &mut Phi, elapsed: f64) -> ViewAction;
    fn render(&self, context: &mut Phi);
}

pub fn spawn<F>(title: &str, init: F)
where F: Fn(&mut Phi) -> Box<View> {
    // Initialize SDL2
    let sdl_context = ::sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let mut timer = sdl_context.timer().unwrap();
    let _image_context = ::sdl2_image::init(::sdl2_image::INIT_PNG).unwrap();
//    let _ttf_context = ::sdl2_ttf::init().unwrap();
    let _mixer_context = ::sdl2_mixer::init(::sdl2_mixer::INIT_OGG).unwrap();

    ::sdl2_mixer::open_audio(44100, ::sdl2_mixer::AUDIO_S16LSB, 2, 1024).unwrap();
    ::sdl2_mixer::allocate_channels(32);

    // Create the window
    let window = video.window("ArcadeRS Shooter", 800, 600)
        .position_centered().opengl().resizable()
        .build().unwrap();

    // Create the context
    let mut context = Phi::new(
        Events::new(sdl_context.event_pump().unwrap()),
        window.renderer()
            .accelerated()
            .build().unwrap());

    // Create the default view
    let mut current_view = init(&mut context);

    // Frame timing
    let interval = 1_000 / 60;
    let mut before = timer.ticks();
    let mut last_second = timer.ticks();
    let mut fps = 0u16;

    loop {
        // Frame timing (bis)

        let now = timer.ticks();
        let dt = now - before;
        let elapsed = dt as f64 / 1_000.0;

        // If the time elapsed since the last frame is too small, wait out the difference and try
        // again
        if dt < interval {
            timer.delay(interval - dt);
            continue;
        }

        before = now;
        fps += 1;

        if now - last_second > 1_000 {
            println!("FPS: {}", fps);
            last_second = now;
            fps = 0;
        }

        // Logic and rendering

        context.events.pump(&mut context.renderer);

        match current_view.update(&mut context, elapsed) {
            ViewAction::Render(view) => {
                current_view = view;
                current_view.render(&mut context);
                context.renderer.present();
            },
            ViewAction::Quit =>
                break,
        }
    }
}

