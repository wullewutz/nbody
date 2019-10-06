//! An attempt to create a n-body simulation that might become a game one day.
//! Inspired by the book "The Three Body Problem" by Liu Cixin.

use ggez;
use ggez::conf;
use ggez::event;
use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics;
use ggez::graphics::{DrawParam};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};

mod galaxy;
use galaxy::Actor;
use galaxy::Point2;
use galaxy::{create_suns, update_vel_and_pos};

const NUMBER_OF_SUNS: u32 = 3;

fn world_to_screen_coords(point: Point2, screen_width: f32, screen_height: f32) -> Point2 {
    let x = point.x + screen_width / 2.0;
    let y = screen_height - (point.y + screen_height / 2.0);
    Point2::new(x, y)
}

fn draw_actor(ctx: &mut Context, actor: &Actor, world_coords: (f32, f32)) -> GameResult {
    let (screen_w, screen_h) = world_coords;
    let pos = world_to_screen_coords(actor.pos, screen_w, screen_h);
    let circle = graphics::Mesh::new_circle(
        ctx,
        graphics::DrawMode::fill(),
        pos,
        actor.radius,
        0.1,
        graphics::Color::new(0.0, 100.0, 0.0, 100.0),
    )?;
    graphics::draw(ctx, &circle, DrawParam::default())
}

struct MainState {
    suns: Vec<Actor>,
    screen_width: f32,
    screen_height: f32,
}

impl MainState {
    fn new(ctx: &mut Context, suns: u32) -> GameResult<MainState> {
        graphics::clear(ctx, (30, 40, 40, 255).into());
        let (width, height) = graphics::drawable_size(ctx);
        let s = MainState {
            suns: create_suns(suns, 300.0),
            screen_width: width,
            screen_height: height,
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
        for s in &self.suns {
            draw_actor(ctx, s, coords).expect("failed to draw a sun");
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
            _ => (), //all other events are unhandled
        }
    }
}

fn main() -> GameResult {
    let cb = ContextBuilder::new("nbody", "wullewutz")
        .window_setup(conf::WindowSetup::default().title("nbody!"))
        .window_mode(conf::WindowMode::default().dimensions(1200.0, 800.0));

    let (ctx, events_loop) = &mut cb.build()?;
    let game = &mut MainState::new(ctx, NUMBER_OF_SUNS).unwrap();
    event::run(ctx, events_loop, game)
}
