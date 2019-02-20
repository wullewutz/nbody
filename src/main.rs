//! An attempt to create a n-body simulation that might become a game one day.
//! Inspired by the book "The Three Body Problem" by Liu Cixin.

extern crate ggez;
extern crate rand;

use ggez::conf;
use ggez::event::{self, EventHandler, Keycode, Mod};
use ggez::graphics;
use ggez::graphics::{DrawMode, Point2, Vector2};
use ggez::nalgebra as na;
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};

fn vec_from_angle(angle: f32) -> Vector2 {
    let x = angle.sin();
    let y = angle.cos();
    Vector2::new(x, y)
}

fn vec_from_points(from: &Point2, to: &Point2) -> Vector2 {
    to.coords - from.coords
}

fn random_vec(max_magnitude: f32) -> Vector2 {
    let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
    let mag = rand::random::<f32>() * max_magnitude;
    vec_from_angle(angle) * (mag)
}

fn world_to_screen_coords(point: Point2, screen_width: u32, screen_height: u32) -> Point2 {
    let width = screen_width as f32;
    let height = screen_height as f32;
    let x = point.x + width / 2.0;
    let y = height - (point.y + height / 2.0);
    Point2::new(x, y)
}

#[derive(Debug, Clone, Copy)]
enum ActorType {
    Sun,
}

#[derive(Debug, Clone, Copy)]
struct Actor {
    tag: ActorType,
    id: u32,
    pos: Point2,
    velocity: Vector2,
    mass: f32,
    radius: f32,
}

const NUMBER_OF_SUNS: u32 = 3;
const SUN_MAX_STARTING_VELOCITY: f32 = 10.0;
const SUN_MIN_MASS: f32 = 10.0;
const SUN_MAX_MASS: f32 = 50.0;
const SUN_DENSITY: f32 = 0.01;
const G: f32 = 6.67384;
const G_DARK: f32 = 0.01;

fn total_momentum(bodys: &Vec<Actor>) -> Vector2 {
    bodys.iter().map(|&b| b.velocity * b.mass).sum()
}

fn total_mass(bodys: &Vec<Actor>) -> f32 {
    bodys.iter().map(|&b| b.mass).sum()
}

fn create_suns(num: u32, galaxy_radius: f32) -> Vec<Actor> {
    let new_sun = |_| {
        let mass = SUN_MIN_MASS + rand::random::<f32>() * (SUN_MAX_MASS - SUN_MIN_MASS);
        let sun = Actor {
            tag: ActorType::Sun,
            id: rand::random::<u32>(),
            pos: Point2::origin() + random_vec(galaxy_radius),
            velocity: random_vec(SUN_MAX_STARTING_VELOCITY),
            mass: mass,
            radius: (mass / SUN_DENSITY * 0.75 / std::f32::consts::PI).cbrt(),
        };
        sun
    };
    let mut suns: Vec<Actor> = (0..num).map(new_sun).collect();

    //Adjust every suns velocity to keep the center of mass in the origin.
    let total_velocity = total_momentum(&suns) / total_mass(&suns);
    for s in &mut suns {
        s.velocity -= total_velocity;
    }
    suns
}

fn elastic_collision(a1: &Actor, a2: &Actor) -> (Vector2, Vector2) {
    fn v_afterwards(this: &Actor, that: &Actor) -> Vector2 {
        this.velocity
            - 2.0 * that.mass / (this.mass + that.mass)
                * (this.velocity - that.velocity).dot(&(this.pos - that.pos))
                / (this.pos - that.pos).norm_squared()
                * (this.pos - that.pos)
    }
    (v_afterwards(a1, a2), v_afterwards(a2, a1))
}

fn update_vel_and_pos(actors: &mut Vec<Actor>, dt: f32) {
    for i in 0..actors.len() {
        for j in 0..actors.len() {
            if i == j {
                continue; //don't apply gravity if both bodys are identical
            }
            //apply gravity
            let r_unit_vec = vec_from_points(&actors[i].pos, &actors[j].pos).normalize();
            let dist_squ = na::distance_squared(&actors[i].pos, &actors[j].pos);
            let g = r_unit_vec * (G * actors[i].mass * actors[j].mass / dist_squ);
            actors[i].velocity += g;

            //check for collision but only once per couple:
            if i < j {
                let touching_dist_squ = (actors[i].radius + actors[j].radius).powf(2.0);
                if dist_squ < touching_dist_squ {
                    let (vi, vj) = elastic_collision(&actors[i], &actors[j]);
                    actors[i].velocity = vi;
                    actors[j].velocity = vj;
                }
            }
        }
        //add little bit of dark matter gravity towards origin to avoid drifting away
        let origin_vec = vec_from_points(&actors[i].pos, &Point2::origin());
        actors[i].velocity += origin_vec * G_DARK;

        //calculate new position of this actor
        let delta_pos = actors[i].velocity * dt;
        actors[i].pos += delta_pos;
    }
}

fn draw_actor(ctx: &mut Context, actor: &Actor) -> GameResult<()> {
    let screen_h = ctx.conf.window_mode.height;
    let screen_w = ctx.conf.window_mode.width;
    let pos = world_to_screen_coords(actor.pos, screen_w, screen_h);
    let color = graphics::Color::new(0.0, 100.0, 0.0, 100.0);
    graphics::set_color(ctx, color).expect("could not set color");
    graphics::circle(ctx, DrawMode::Line(1.0), pos, actor.radius, 0.1)
}

struct MainState {
    suns: Vec<Actor>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        ctx.print_resource_stats();
        graphics::set_background_color(ctx, (30, 40, 40, 255).into());
        let s = MainState {
            suns: create_suns(NUMBER_OF_SUNS, 300.0),
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
    let cb = ContextBuilder::new("n-body", "lordwuwu")
        .window_setup(conf::WindowSetup::default().title("n-body!"))
        .window_mode(conf::WindowMode::default().dimensions(1200, 800));

    let ctx = &mut cb.build().unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_collision_central_one_moving() {
        let mut a = Actor {
            tag: ActorType::Sun,
            id: 1,
            pos: Point2::new(0.0, 0.0),
            velocity: Vector2::new(10.0, 0.0),
            mass: 10.0,
            radius: 100.0,
        };
        let mut b = Actor {
            tag: ActorType::Sun,
            id: 2,
            pos: Point2::new(200.0, 0.0),
            velocity: Vector2::new(0.0, 0.0),
            mass: 10.0,
            radius: 100.0,
        };
        let (v1, v2) = elastic_collision(&mut a, &mut b);
        //test if both velocities have swaped.
        assert_approx_eq!(v1.x, 0.0);
        assert_approx_eq!(v2.x, 10.0);
    }

    #[test]
    fn test_collision_central_both_moving() {
        let mut a = Actor {
            tag: ActorType::Sun,
            id: 1,
            pos: Point2::new(0.0, 0.0),
            velocity: Vector2::new(10.0, 0.0),
            mass: 10.0,
            radius: 100.0,
        };
        let mut b = Actor {
            tag: ActorType::Sun,
            id: 2,
            pos: Point2::new(200.0, 0.0),
            velocity: Vector2::new(-10.0, 0.0),
            mass: 10.0,
            radius: 100.0,
        };
        let (v1, v2) = elastic_collision(&mut a, &mut b);
        //test if both velocities have swaped.
        assert_approx_eq!(v1.x, -10.0);
        assert_approx_eq!(v2.x, 10.0);
    }
}
