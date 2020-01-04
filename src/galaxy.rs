use ggez::nalgebra as na;
use itertools::Itertools;
use std::collections::VecDeque;

pub type Point2 = na::Point2<f32>;
type Vector2 = na::Vector2<f32>;

//Star class taken from table at
//https://de.wikipedia.org/wiki/Klassifizierung_der_Sterne
const CLASS_O: f32 = 60.0;
const CLASS_B: f32 = 18.0;
const CLASS_A: f32 = 3.2;
const CLASS_F: f32 = 1.7;
const CLASS_G: f32 = 1.1;
const CLASS_K: f32 = 0.8;
const CLASS_M: f32 = 0.3;

const G: f32 = 1_000.0;
const SUN_MAX_STARTING_VELOCITY: f32 = 100.0;
const SUN_MIN_MASS: f32 = CLASS_M;
const SUN_MAX_MASS: f32 = CLASS_O;
const SUN_DENSITY: f32 = 0.002; // higher density -> smaller radius

const TRACE_LEN: usize = 600; // number of points to be drawn as the body's path.

#[derive(Debug, Clone, Copy)]
enum ActorType {
    Sun,
}

#[derive(Debug, Clone)]
pub struct Actor {
    tag: ActorType,
    id: u32,
    pub pos: Point2,
    pub trace: VecDeque<Point2>,
    trace_cnt: u32,
    pub radius: f32,
    velocity: Vector2,
    new_velocity: Vector2,
    mass: f32,
    pub color: u32,
}

fn color_from_mass(mass: f32) -> u32 {
    if mass < CLASS_M {
        0xfbc8_86ff
    } else if mass < CLASS_K {
        0xffd8_70ff
    } else if mass < CLASS_G {
        0xfdf9_b3ff
    } else if mass < CLASS_F {
        0xf9fa_e7ff
    } else if mass < CLASS_A {
        0xdadd_e6ff
    } else if mass < CLASS_B {
        0xaabf_ffff
    } else if mass < CLASS_O {
        0x9bb0_ffff
    } else {
        0xffff_ffff
    }
}

fn vec_from_angle(angle: f32) -> Vector2 {
    let x = angle.sin();
    let y = angle.cos();
    Vector2::new(x, y)
}

fn vec_from_points(from: Point2, to: Point2) -> Vector2 {
    to.coords - from.coords
}

fn random_vec(max_magnitude: f32) -> Vector2 {
    let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
    let mag = rand::random::<f32>() * max_magnitude;
    vec_from_angle(angle) * (mag)
}

fn total_momentum(bodys: &[Actor]) -> Vector2 {
    bodys.iter().map(|b| b.velocity * b.mass).sum()
}

fn total_mass(bodys: &[Actor]) -> f32 {
    bodys.iter().map(|b| b.mass).sum()
}

pub fn create_suns(num: u32, galaxy_radius: f32) -> Vec<Actor> {
    let new_sun = |_| {
        let m = SUN_MIN_MASS + rand::random::<f32>().powf(10.0) * (SUN_MAX_MASS - SUN_MIN_MASS);
        Actor {
            tag: ActorType::Sun,
            id: rand::random::<u32>(),
            pos: Point2::origin() + random_vec(galaxy_radius),
            trace: VecDeque::with_capacity(TRACE_LEN),
            trace_cnt: 0,
            velocity: random_vec(SUN_MAX_STARTING_VELOCITY),
            new_velocity: Vector2::new(0.0, 0.0),
            mass: m,
            radius: (m / SUN_DENSITY * 0.75 / std::f32::consts::PI).cbrt(),
            color: color_from_mass(m),
        }
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

pub fn update_vel_and_pos(actors: &mut Vec<Actor>, dt: f32) {
    for (a, b) in (0..actors.len()).tuple_combinations() {
        let r_unit_vec = vec_from_points(actors[a].pos, actors[b].pos).normalize();
        let dist_squ = na::distance_squared(&actors[a].pos, &actors[b].pos);
        // check for collision
        let touching_dist_squ = (actors[a].radius + actors[b].radius).powf(2.0);
        if dist_squ < touching_dist_squ {
            let (va, vb) = elastic_collision(&actors[a], &actors[b]);
            actors[a].new_velocity = va;
            actors[b].new_velocity = vb;
        } else {
            //apply gravity force fg
            let fg = r_unit_vec * (G * actors[a].mass * actors[b].mass / dist_squ);
            let delta_vg_a = fg / actors[a].mass;
            let delta_vg_b = -fg / actors[b].mass;
            actors[a].new_velocity += delta_vg_a;
            actors[b].new_velocity += delta_vg_b;
        }
    }
    //calculate new position for every actor
    for a in actors.into_iter() {
        a.velocity = a.new_velocity;
        a.pos += a.velocity * dt;
        a.trace_cnt += 1;
        if a.trace_cnt == 10 {
            a.trace_cnt = 0;
            a.trace.push_front(a.pos);
            if a.trace.len() >= TRACE_LEN {
                a.trace.pop_back();
            }
        }
    }
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
            trace: VecDeque::new(),
            trace_cnt: 0,
            radius: 100.0,
            velocity: Vector2::new(10.0, 0.0),
            new_velocity: Vector2::new(0.0, 0.0),
            mass: 10.0,
            color: 0x0000_0000,
        };
        let mut b = Actor {
            tag: ActorType::Sun,
            id: 2,
            pos: Point2::new(200.0, 0.0),
            trace: VecDeque::new(),
            trace_cnt: 0,
            radius: 100.0,
            velocity: Vector2::new(0.0, 0.0),
            new_velocity: Vector2::new(0.0, 0.0),
            mass: 10.0,
            color: 0x0000_0000,
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
            trace: VecDeque::new(),
            trace_cnt: 0,
            radius: 100.0,
            velocity: Vector2::new(10.0, 0.0),
            new_velocity: Vector2::new(0.0, 0.0),
            mass: 10.0,
            color: 0x0000_0000,
        };
        let mut b = Actor {
            tag: ActorType::Sun,
            id: 2,
            pos: Point2::new(200.0, 0.0),
            trace: VecDeque::new(),
            trace_cnt: 0,
            radius: 100.0,
            velocity: Vector2::new(-10.0, 0.0),
            new_velocity: Vector2::new(0.0, 0.0),
            mass: 10.0,
            color: 0x0000_0000,
        };
        let (v1, v2) = elastic_collision(&mut a, &mut b);
        //test if both velocities have swaped.
        assert_approx_eq!(v1.x, -10.0);
        assert_approx_eq!(v2.x, 10.0);
    }
}
