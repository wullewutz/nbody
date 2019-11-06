use ggez;
use ggez::conf;
use ggez::event;
use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics;
use ggez::graphics::DrawParam;
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};

use super::galaxy::Actor;
use super::galaxy::Point2;
use super::galaxy::{create_suns, update_vel_and_pos};

const ZOOM_FACTOR: f32 = 1.2;

struct MainState {
    suns: Vec<Actor>,
    screen_width: f32,
    screen_height: f32,
    zoom: f32,
    zoom_target: f32,
}

pub fn start(suns :u32) -> GameResult {
    let cb = ContextBuilder::new("nbody", "wullewutz")
        .window_setup(conf::WindowSetup::default().title("nbody!"))
        .window_mode(conf::WindowMode::default().dimensions(1200.0, 800.0));

    let (ctx, events_loop) = &mut cb.build()?;
    let game = &mut MainState::new(ctx, suns)?;
    event::run(ctx, events_loop, game)
}

fn world_to_screen_coords(
    point: Point2,
    screen_width: f32,
    screen_height: f32,
    zoom: f32,
) -> Point2 {
    let x = point.x * zoom + screen_width / 2.0;
    let y = screen_height - (point.y * zoom + screen_height / 2.0);
    Point2::new(x, y)
}

fn zoom_smooth(zoom_current: f32, zoom_target: f32) -> f32 {
    const ZOOM_SMOOTH: f32 = 0.1;
    if (zoom_target - zoom_current).abs() < zoom_target / 1000.0 {
        zoom_target
    } else {
        zoom_current + (zoom_target - zoom_current) * ZOOM_SMOOTH
    }
}

fn draw_actor(ctx: &mut Context, actor: &Actor, world_coords: (f32, f32), zoom: f32) -> GameResult {
    let (screen_w, screen_h) = world_coords;
    let pos = world_to_screen_coords(actor.pos, screen_w, screen_h, zoom);
    let circle = graphics::Mesh::new_circle(
        ctx,
        graphics::DrawMode::fill(),
        pos,
        // Raius + 1.0 in order to still draws actor for very far-out zooms.
        actor.radius * zoom + 1.0,
        0.05 * actor.radius / zoom,
        graphics::Color::from_rgba_u32(actor.color),
    )?;
    graphics::draw(ctx, &circle, DrawParam::default())
}

impl MainState {
    fn new(ctx: &mut Context, suns: u32) -> GameResult<MainState> {
        graphics::clear(ctx, (30, 40, 40, 255).into());
        let (width, height) = graphics::drawable_size(ctx);
        let initial_zoom = 1.0;
        let s = MainState {
            suns: create_suns(suns, height / 5.0 * suns as f32 ),
            screen_width: width,
            screen_height: height,
            zoom: initial_zoom,
            zoom_target: initial_zoom,
        };
        Ok(s)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        let dt = 1.0 / (DESIRED_FPS as f32);

        while timer::check_update_time(ctx, DESIRED_FPS) {
            update_vel_and_pos(&mut self.suns, dt);
            //println!("{}", timer::fps(ctx));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, (30, 40, 40, 255).into());
        let coords = (self.screen_width, self.screen_height);
        self.zoom = zoom_smooth(self.zoom, self.zoom_target);
        for s in &self.suns {
            draw_actor(ctx, s, coords, self.zoom).expect("failed to draw a sun");
        }
        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Escape | KeyCode::Q => event::quit(ctx),
            KeyCode::I => self.zoom_target *= ZOOM_FACTOR,
            KeyCode::O => self.zoom_target /= ZOOM_FACTOR,
            _ => (), //all other events are unhandled
        }
    }
}
