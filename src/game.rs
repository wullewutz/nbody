use std::collections::VecDeque;

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

const SCREEN_W: f32 = 1200.0;
const SCREEN_H: f32 = 800.0;

const ZOOM_FACTOR: f32 = 1.2;
const SPEED_FACTOR: f32 = 2.0;
const MOVE_DELTA: f32 = SCREEN_W / 10.0;

struct MainState {
    suns: Vec<Actor>,
    screen_width: f32,
    screen_height: f32,
    center: Point2,
    center_target: Point2,
    zoom: f32,
    zoom_target: f32,
    speed: f32,
    running: bool,
    show_traces: bool,
}

pub fn start(suns: u32) -> GameResult {
    let cb = ContextBuilder::new("nbody", "wullewutz")
        .window_setup(conf::WindowSetup::default().title("nbody!"))
        .window_mode(conf::WindowMode::default().dimensions(SCREEN_W, SCREEN_H));

    let (ctx, events_loop) = &mut cb.build()?;
    let game = &mut MainState::new(ctx, suns)?;
    event::run(ctx, events_loop, game)
}

fn world_to_screen_coords(
    point: Point2,
    screen_width: f32,
    screen_height: f32,
    zoom: f32,
    center: Point2,
) -> Point2 {
    let x = (point.x - center.x) * zoom + screen_width / 2.0;
    let y = -(point.y - center.y) * zoom + screen_height / 2.0;
    Point2::new(x, y)
}

fn zoom_smooth(zoom_current: f32, zoom_target: f32) -> f32 {
    const ZOOM_SMOOTH: f32 = 0.1;
    zoom_current + (zoom_target - zoom_current) * ZOOM_SMOOTH
}

fn move_smooth(center_current: Point2, center_target: Point2) -> Point2 {
    const MOVE_SMOOTH: f32 = 0.1;
    Point2::new(
        center_current.x + MOVE_SMOOTH * (center_target.x - center_current.x),
        center_current.y + MOVE_SMOOTH * (center_target.y - center_current.y),
    )
}

fn draw_actor(
    ctx: &mut Context,
    actor: &Actor,
    world_coords: (f32, f32),
    zoom: f32,
    center: Point2,
) -> GameResult {
    let (screen_w, screen_h) = world_coords;
    let pos = world_to_screen_coords(actor.pos, screen_w, screen_h, zoom, center);
    let circle = graphics::Mesh::new_circle(
        ctx,
        graphics::DrawMode::fill(),
        pos,
        // Radius + 1.0 in order to still draws actor for very far-out zooms.
        actor.radius * zoom + 1.0,
        0.05 * actor.radius / zoom,
        graphics::Color::from_rgba_u32(actor.color),
    )?;
    graphics::draw(ctx, &circle, DrawParam::default())
}

fn draw_trace(
    ctx: &mut Context,
    trace: &VecDeque<Point2>,
    color: u32,
    world_coords: (f32, f32),
    zoom: f32,
    center: Point2,
) -> GameResult {
    if trace.len() >= 3 {
        let (screen_w, screen_h) = world_coords;
        let mut t = Vec::new();
        for p in trace {
            t.push(world_to_screen_coords(*p, screen_w, screen_h, zoom, center));
        }
        let trace_line =
            graphics::Mesh::new_line(ctx, &t, 1.0, graphics::Color::from_rgba_u32(color))?;
        graphics::draw(ctx, &trace_line, DrawParam::default())
    } else {
        Ok(())
    }
}

impl MainState {
    fn new(ctx: &mut Context, suns: u32) -> GameResult<MainState> {
        graphics::clear(ctx, (30, 40, 40, 255).into());
        let (width, height) = graphics::drawable_size(ctx);
        let s = MainState {
            suns: create_suns(suns, height / 20.0 * suns as f32),
            screen_width: width,
            screen_height: height,
            center: Point2::origin(),
            center_target: Point2::origin(),
            zoom: 1.0,
            zoom_target: 1.0,
            speed: 1.0,
            running: true,
            show_traces: true,
        };
        Ok(s)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;
        let dt = self.speed / (DESIRED_FPS as f32);
        while timer::check_update_time(ctx, DESIRED_FPS) {
            if self.running {
                update_vel_and_pos(&mut self.suns, dt);
            }
            // println!("{}", timer::fps(ctx));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, (30, 40, 40, 255).into());
        let coords = (self.screen_width, self.screen_height);
        self.zoom = zoom_smooth(self.zoom, self.zoom_target);
        self.center = move_smooth(self.center, self.center_target);
        for s in &self.suns {
            if self.show_traces {
                draw_trace(ctx, &s.trace, s.color, coords, self.zoom, self.center)
                    .expect("failed to draw trace");
            }
            draw_actor(ctx, s, coords, self.zoom, self.center).expect("failed to draw a sun");
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
            KeyCode::Space => self.running = !self.running,
            KeyCode::Add => self.speed *= SPEED_FACTOR,
            KeyCode::Subtract => self.speed /= SPEED_FACTOR,
            KeyCode::I => self.zoom_target *= ZOOM_FACTOR,
            KeyCode::O => self.zoom_target /= ZOOM_FACTOR,
            KeyCode::A => self.center_target.x -= MOVE_DELTA / self.zoom,
            KeyCode::D => self.center_target.x += MOVE_DELTA / self.zoom,
            KeyCode::S => self.center_target.y -= MOVE_DELTA / self.zoom,
            KeyCode::W => self.center_target.y += MOVE_DELTA / self.zoom,
            KeyCode::T => self.show_traces = !self.show_traces,
            _ => (), //all other events are unhandled
        }
    }
}
