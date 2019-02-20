//! An attempt to create a n-body simulation that might become a game one day.
//! Inspired by the book "The Three Body Problem" by Liu Cixin.

use ggez::conf;
use ggez::event::{self, EventHandler, Keycode, Mod};
use ggez::graphics;
use ggez::graphics::{DrawMode, Point2};
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};

mod galaxy;
use galaxy::Actor;
use galaxy::{create_suns, update_vel_and_pos};

const NUMBER_OF_SUNS: u32 = 3;

fn world_to_screen_coords(point: Point2, screen_width: u32, screen_height: u32) -> Point2 {
    let width = screen_width as f32;
    let height = screen_height as f32;
    let x = point.x + width / 2.0;
    let y = height - (point.y + height / 2.0);
    Point2::new(x, y)
}

fn draw_actor(ctx: &mut Context, actor: &Actor) -> GameResult<()> {
    let screen_h = ctx.conf.window_mode.height;
    let screen_w = ctx.conf.window_mode.width;
    let pos = world_to_screen_coords(actor.pos, screen_w, screen_h);
    let color = graphics::Color::new(0.0, 100.0, 0.0, 100.0);
    graphics::set_color(ctx, color).expect("could not set color");
    graphics::circle(ctx, DrawMode::Fill, pos, actor.radius, 0.1)
}

struct MainState {
    suns: Vec<Actor>,
}

impl MainState {
    fn new(ctx: &mut Context, suns: u32) -> GameResult<MainState> {
        ctx.print_resource_stats();
        graphics::set_background_color(ctx, (30, 40, 40, 255).into());
        let s = MainState {
            suns: create_suns(suns, 300.0),
        };
        Ok(s)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 120;
        let dt = 1.0 / (DESIRED_FPS as f32);

        while timer::check_update_time(ctx, DESIRED_FPS) {
            update_vel_and_pos(&mut self.suns, dt);
            //println!("{}", timer::get_fps(ctx));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        for s in &self.suns {
            draw_actor(ctx, s).expect("failed to draw a sun");
        }
        graphics::present(ctx);
        timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::Escape | Keycode::Q => ctx.quit().unwrap(),
            _ => (), //all other events are unhandled
        }
    }
}

fn main() {
    let cb = ContextBuilder::new("nbody", "wullewutz")
        .window_setup(conf::WindowSetup::default().title("nbody!"))
        .window_mode(conf::WindowMode::default().dimensions(1200, 800));

    let ctx = &mut cb.build().unwrap();
    let state = &mut MainState::new(ctx, NUMBER_OF_SUNS).unwrap();
    event::run(ctx, state).unwrap();
}
