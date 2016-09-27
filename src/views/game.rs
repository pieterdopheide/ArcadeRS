use phi::{Phi, View, ViewAction};
use phi::data::{MaybeAlive, Rectangle};
use phi::gfx::{AnimatedSprite, CopySprite, Sprite};
use sdl2::pixels::Color;
use views::shared::BgSet;

const PLAYER_SPEED: f64 = 180.0;

const SHIP_W: f64 = 43.0;
const SHIP_H: f64 = 39.0;

const BULLET_SPEED: f64 = 240.0;
const BULLET_W: f64 = 8.0;
const BULLET_H: f64 = 4.0;

const ASTEROID_PATH: &'static str = "assets/asteroid.png";
const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROID_SIDE: f64 = 96.0;

const DEBUG: bool = false;

#[derive(Clone, Copy)]
enum ShipFrame {
    UpNorm = 0,
    UpFast = 1,
    UpSlow = 2,
    MidNorm = 3,
    MidFast = 4,
    MidSlow = 5,
    DownNorm = 6,
    DownFast = 7,
    DownSlow = 8
}

#[derive(Clone, Copy)]
enum CannonType {
    RectBullet,
    SineBullet { amplitude: f64, angular_vel: f64 },
    DivergentBullet { a: f64, b: f64 },
}

struct SineBullet {
    pos_x: f64,
    origin_y: f64,
    amplitude: f64,
    angular_vel: f64,
    total_time: f64,
}

impl Bullet for SineBullet {
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        self.total_time += dt;
        self.pos_x += BULLET_SPEED * dt;

        let (w, _) = phi.output_size();

        if self.rect().x > w {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
    }

    fn rect(&self) -> Rectangle {
        let dy = self.amplitude * f64::sin(self.angular_vel * self.total_time);
        Rectangle {
            x: self.pos_x,
            y: self.origin_y + dy,
            w: BULLET_W,
            h: BULLET_H,
        }
    }
}

// Bullet which follows a vertical trajectory given by:
// a * ((t / b)^3 - (t / b)^2)
struct DivergentBullet {
    pos_x: f64,
    origin_y: f64,
    a: f64, // Influences the bump's height
    b: f64, // Influences the bump's width
    total_time: f64,
}

impl Bullet for DivergentBullet {
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        self.total_time += dt;
        self.pos_x += BULLET_SPEED * dt;

        let (w, h) = phi.output_size();
        let rect = self.rect();

        if rect.x > w || rect.x < 0.0 ||
           rect.y > h || rect.y < 0.0 {
                None
            } else {
                Some(self)
            }
    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
    }

    fn rect(&self) -> Rectangle {
        let dy = self.a * 
            ((self.total_time / self.b).powi(3) -
             (self.total_time / self.b).powi(2));

        Rectangle {
            x: self.pos_x,
            y: self.origin_y + dy,
            w: BULLET_W,
            h: BULLET_H,
        }
    }
}

struct Ship {
    rect: Rectangle,
    sprites: Vec<Sprite>,
    current: ShipFrame,
    cannon: CannonType,
}

impl Ship {
    fn spawn_bullets(&self) -> Vec<Box<Bullet>> {
        let cannons_x = self.rect.x + 30.0;
        let cannon1_y = self.rect.y + 6.0;
        let cannon2_y = self.rect.y + SHIP_H - 10.0;

        match self.cannon {
            CannonType::RectBullet =>
                vec![
                    Box::new(RectBullet {
                        rect: Rectangle {
                            x: cannons_x,
                            y: cannon1_y,
                            w: BULLET_W,
                            h: BULLET_H,
                        }
                    }),
                    Box::new(RectBullet {
                        rect: Rectangle {
                            x: cannons_x,
                            y: cannon2_y,
                            w: BULLET_W,
                            h: BULLET_H,
                        }
                    }),
                ],

            CannonType::SineBullet { amplitude, angular_vel } =>
                vec![
                    Box::new(SineBullet {
                        pos_x: cannons_x,
                        origin_y: cannon1_y,
                        amplitude: amplitude,
                        angular_vel: angular_vel,
                        total_time: 0.0,
                    }),
                    Box::new(SineBullet {
                        pos_x: cannons_x,
                        origin_y: cannon2_y,
                        amplitude: amplitude,
                        angular_vel: angular_vel,
                        total_time: 0.0,
                    }),
                ],

            CannonType::DivergentBullet { a, b } =>
                vec![
                    Box::new(DivergentBullet {
                        pos_x: cannons_x,
                        origin_y: cannon1_y,
                        a: -a,
                        b: b,
                        total_time: 0.0,
                    }),
                    Box::new(DivergentBullet {
                        pos_x: cannons_x,
                        origin_y: cannon2_y,
                        a: a,
                        b: b,
                        total_time: 0.0,
                    }),
                ]
        }
    }
}

trait Bullet {
    // Update the bullet
    fn update(self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>>;

    // Render the bullet to the screen
    fn render(&self, phi: &mut Phi);

    // Return the bullet's bounding box
    fn rect(&self) -> Rectangle;
}

struct RectBullet {
    rect: Rectangle,
}

impl RectBullet {
    fn new(x: f64, y: f64) -> RectBullet {
        RectBullet {
            rect: Rectangle {
                x: x,
                y: y,
                w: BULLET_W,
                h: BULLET_H,
            }
        }
    }
}

impl Bullet for RectBullet {
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        let (w, _) = phi.output_size();
        self.rect.x += BULLET_SPEED * dt;

        // If the bullet has left the screen, then delete it
        if self.rect.x > w {
            None
        } else {
            Some(self)
        }
    }

    // Render the bullet to the screen
    fn render(&self, phi: &mut Phi) {
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30)); // bullet is yellow
        phi.renderer.fill_rect(self.rect.to_sdl().unwrap());
    }

    // Return the bullet's bounding box
    fn rect(&self) -> Rectangle {
        self.rect
    }
}

struct Asteroid {
    sprite: AnimatedSprite,
    rect: Rectangle,
    vel: f64,
}

impl Asteroid {
/*
    fn new(phi: &mut Phi) -> Asteroid {
        let mut asteroid = 
            Asteroid {
                sprite: Asteroid::get_sprite(phi, 15.0),
                rect: Rectangle {
                    w: 0.0,
                    h: 0.0,
                    x: 0.0,
                    y: 0.0,
                },
                vel: 0.0,
            };

        asteroid.reset(phi);
        asteroid
    }
*/

    fn factory(phi: &mut Phi) -> AsteroidFactory {
        let asteroid_spritesheet = Sprite::load(&mut phi.renderer, ASTEROID_PATH).unwrap();
        let mut asteroid_sprites = Vec::with_capacity(ASTEROIDS_TOTAL);

        for yth in 0..ASTEROIDS_HIGH {
            for xth in 0..ASTEROIDS_WIDE {
                if ASTEROIDS_WIDE * yth + xth >= ASTEROIDS_TOTAL {
                    break;
                }

                asteroid_sprites.push(
                    asteroid_spritesheet.region(Rectangle {
                        w: ASTEROID_SIDE,
                        h: ASTEROID_SIDE,
                        x: ASTEROID_SIDE * xth as f64,
                        y: ASTEROID_SIDE * yth as f64,
                    }).unwrap());
            }
        }

        AsteroidFactory {
            sprite: AnimatedSprite::with_fps(asteroid_sprites, 1.0),
        }
    }

    fn reset(&mut self, phi: &mut Phi) {
        let (w, h) = phi.output_size();

        self.sprite.set_fps(::rand::random::<f64>().abs() * 20.0 + 10.0);

        self.rect = Rectangle {
            w: ASTEROID_SIDE,
            h: ASTEROID_SIDE,
            x: w,
            y: ::rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
        };

        self.vel = ::rand::random::<f64>().abs() * 100.0 + 50.0;
    }

/*
    fn get_sprite(phi: &mut Phi, fps: f64) -> AnimatedSprite {
        let asteroid_spritesheet = Sprite::load(&mut phi.renderer, ASTEROID_PATH).unwrap();
        let mut asteroid_sprites = Vec::with_capacity(ASTEROIDS_TOTAL);

        for yth in 0..ASTEROIDS_HIGH {
            for xth in 0..ASTEROIDS_WIDE {
                if ASTEROIDS_WIDE * yth + xth >= ASTEROIDS_TOTAL {
                    break;
                }

                asteroid_sprites.push(
                    asteroid_spritesheet.region(Rectangle {
                        w: ASTEROID_SIDE,
                        h: ASTEROID_SIDE,
                        x: ASTEROID_SIDE * xth as f64,
                        y: ASTEROID_SIDE * yth as f64,
                    }).unwrap());
            }
        }

        AnimatedSprite::with_fps(asteroid_sprites, fps)
    }
*/

    fn update(mut self, dt: f64) -> Option<Asteroid> {
        self.rect.x -= dt * self.vel;
        self.sprite.add_time(dt);

        if self.rect.x <= -ASTEROID_SIDE {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        if DEBUG {
            // Render the bounding box
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
        }

        phi.renderer.copy_sprite(&self.sprite, self.rect);
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }
}

struct AsteroidFactory {
    sprite: AnimatedSprite,
}

impl AsteroidFactory {
    fn random(&self, phi: &mut Phi) -> Asteroid {
        let (w, h) = phi.output_size();

        // FPS in [10.0, 30.0]
        let mut sprite = self.sprite.clone();
        sprite.set_fps(::rand::random::<f64>().abs() * 20.0 + 10.0);

        Asteroid {
            sprite: sprite,

            rect: Rectangle {
                w: ASTEROID_SIDE,
                h: ASTEROID_SIDE,
                x: w,
                y: ::rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
            },

            // vel in [50.0, 150.0]
            vel: ::rand::random::<f64>().abs() * 100.0 + 50.0,
        }
    }
}

pub struct GameView {
    player: Ship,
    bullets: Vec<Box<Bullet>>,
    asteroids: Vec<Asteroid>,
    asteroid_factory: AsteroidFactory,
    bg: BgSet,
}

impl GameView {
    // We temporarily keep this so that we can instanciate 'GameView' in
    // 'main' while developing it further
    #[allow(dead_code)]
    pub fn new(phi: &mut Phi) -> GameView {
        let bg = BgSet::new(&mut phi.renderer);
        GameView::with_backgrounds(phi, bg)
    }

    pub fn with_backgrounds(phi: &mut Phi, bg: BgSet) -> GameView {
        let spritesheet = Sprite::load(&mut phi.renderer, "assets/spaceship.png").unwrap();
        let mut sprites = Vec::with_capacity(9);

        for y in 0..3 {
            for x in 0..3 {
                sprites.push(spritesheet.region(Rectangle {
                    w: SHIP_W,
                    h: SHIP_H,
                    x: SHIP_W * x as f64,
                    y: SHIP_H * y as f64,
                }).unwrap());
            }
        }

        GameView {
            player: Ship {
                rect: Rectangle {
                    x: 64.0,
                    y: 64.0,
                    w: SHIP_W,
                    h: SHIP_H,
                },
                sprites: sprites,
                current: ShipFrame::MidNorm,
                cannon: CannonType::RectBullet, // RectBullet is default bullet type
            },

            bullets: vec![],
            asteroids: vec![],
            asteroid_factory: Asteroid::factory(phi),
            bg: bg,
        }
    }
}

impl View for GameView {
    fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
        if phi.events.now.quit {
            return ViewAction::Quit;
        }

        if phi.events.now.key_escape == Some(true) {
            return ViewAction::ChangeView(Box::new(
                    ::views::main_menu::MainMenuView::with_backgrounds(
                        phi, self.bg.clone())));
        }

        // Change player's cannons

        if phi.events.now.key_1 == Some(true) {
            self.player.cannon = CannonType::RectBullet;
        }

        if phi.events.now.key_2 == Some(true) {
            self.player.cannon = CannonType::SineBullet {
                amplitude: 10.0,
                angular_vel: 15.0,
            };
        }

        if phi.events.now.key_3 == Some(true) {
            self.player.cannon = CannonType::DivergentBullet {
                a: 100.0,
                b: 1.2,
            };
        }

        // Move the player's ship

        let diagonal = (phi.events.key_up ^ phi.events.key_down) && (phi.events.key_left ^ phi.events.key_right);

        let moved = if diagonal { 1.0 / 2.0f64.sqrt() } else { 1.0 } * PLAYER_SPEED * elapsed;

        let dx = match (phi.events.key_left, phi.events.key_right) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -moved,
            (false, true) => moved,
        };

        let dy = match (phi.events.key_up, phi.events.key_down) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -moved,
            (false, true) => moved,
        };

        self.player.rect.x += dx;
        self.player.rect.y += dy;

        let movable_region = Rectangle {
            x: 0.0,
            y: 0.0,
            w: phi.output_size().0 as f64 * 0.70,
            h: phi.output_size().1 as f64,
        };

        self.player.rect = self.player.rect.move_inside(movable_region).unwrap();

        // Select the appropriate sprite of the ship to show
        self.player.current =
            if dx == 0.0 && dy < 0.0 { ShipFrame::UpNorm }
            else if dx > 0.0 && dy < 0.0 { ShipFrame::UpFast }
            else if dx < 0.0 && dy < 0.0 { ShipFrame::UpSlow }
            else if dx == 0.0 && dy == 0.0 { ShipFrame::MidNorm }
            else if dx > 0.0 && dy == 0.0 { ShipFrame::MidFast }
            else if dx < 0.0 && dy == 0.0 { ShipFrame::MidSlow }
            else if dx == 0.0 && dy > 0.0 { ShipFrame::DownNorm }
            else if dx > 0.0 && dy > 0.0 { ShipFrame::DownFast }
            else if dx < 0.0 && dy > 0.0 { ShipFrame::DownSlow }
            else { unreachable!() };

        // Update the bullets
        let old_bullets = ::std::mem::replace(&mut self.bullets, vec![]);
        self.bullets = old_bullets.into_iter()
            .filter_map(|bullet| bullet.update(phi, elapsed))
            .collect();

        // Update the asteroid
        self.asteroids =
            ::std::mem::replace(&mut self.asteroids, vec![])
            .into_iter()
            .filter_map(|asteroid| asteroid.update(elapsed))
            .collect();

        // Collision detection

        let mut player_alive = true;

        let mut transition_bullets: Vec<_> =
            ::std::mem::replace(&mut self.bullets, vec![])
            .into_iter()
            .map(|bullet| MaybeAlive { alive: true, value: bullet })
            .collect();

        self.asteroids =
            ::std::mem::replace(&mut self.asteroids, vec![])
            .into_iter()
            .filter_map(|asteroid| {
                let mut asteroid_alive = true;

                for bullet in &mut transition_bullets {
                    if asteroid.rect().overlaps(bullet.value.rect()) {
                        asteroid_alive = false;
                        bullet.alive = false;
                    }
                }

                if asteroid.rect().overlaps(self.player.rect) {
                    asteroid_alive = false;
                    player_alive = false;
                }

                if asteroid_alive {
                    Some(asteroid)
                } else {
                    None
                }
            })
            .collect();

        self.bullets = transition_bullets.into_iter()
            .filter_map(MaybeAlive::as_option)
            .collect();

        if !player_alive {
            println!("The player's ship has been destroid");
        }

        // Allow the player to shoot after the bullets are updated
        if phi.events.now.key_space == Some(true) {
            self.bullets.append(&mut self.player.spawn_bullets());
        }

        // Randomly create an asteroid about once every 100 frames, ~2 seconds
        if ::rand::random::<usize>() % 100 == 0 {
            self.asteroids.push(self.asteroid_factory.random(phi));
        }

        // Clear the screen
        phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
        phi.renderer.clear();

        // Render the backgrounds
        self.bg.back.render(&mut phi.renderer, elapsed);
        self.bg.middle.render(&mut phi.renderer, elapsed);

        // Render the bounding box (for debugging purposes)
        if DEBUG {
            // Render the scene
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            phi.renderer.fill_rect(self.player.rect.to_sdl().unwrap());
        }

        // Render the ship
        phi.renderer.copy_sprite(&self.player.sprites[self.player.current as usize], self.player.rect);

        // Render the bullets
        for bullet in &self.bullets {
            bullet.render(phi);
        }

        // Render the asteroid
        for asteroid in &self.asteroids {
            asteroid.render(phi);
        }

        // Render the foreground
        self.bg.front.render(&mut phi.renderer, elapsed);

        ViewAction::None
    }
}

