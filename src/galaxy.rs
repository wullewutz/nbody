use ggez::nalgebra as na;

pub type Point2 = na::Point2<f32>;
type Vector2 = na::Vector2<f32>;

const SUN_MAX_STARTING_VELOCITY: f32 = 20.0;
const SUN_MIN_MASS: f32 = 10.0;
const SUN_MAX_MASS: f32 = 50.0;
const SUN_DENSITY: f32 = 0.02; // higher density -> smaller radius
const G: f32 = 1.0;
const G_DARK: f32 = G / 1000.0;

#[derive(Debug, Clone, Copy)]
enum ActorType {
    Sun,
}

#[derive(Debug, Clone, Copy)]
pub struct Actor {
    tag: ActorType,
    id: u32,
    pub pos: Point2,
    pub radius: f32,
    velocity: Vector2,
    mass: f32,
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
    bodys.iter().map(|&b| b.velocity * b.mass).sum()
}

fn total_mass(bodys: &[Actor]) -> f32 {
    bodys.iter().map(|&b| b.mass).sum()
}

pub fn create_suns(num: u32, galaxy_radius: f32) -> Vec<Actor> {
    let new_sun = |_| {
        let m = SUN_MIN_MASS + rand::random::<f32>() * (SUN_MAX_MASS - SUN_MIN_MASS);
        Actor {
            tag: ActorType::Sun,
            id: rand::random::<u32>(),
            pos: Point2::origin() + random_vec(galaxy_radius),
            velocity: random_vec(SUN_MAX_STARTING_VELOCITY),
            mass: m,
            radius: (m / SUN_DENSITY * 0.75 / std::f32::consts::PI).cbrt(),
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
                / (this.pos - that.pos).norm_squared() * (this.pos - that.pos)
    }
    (v_afterwards(a1, a2), v_afterwards(a2, a1))
}

pub fn update_vel_and_pos(actors: &mut Vec<Actor>, dt: f32) {
    for i in 0..actors.len() {
        for j in 0..actors.len() {
            if i == j {
                continue; //don't apply gravity if both bodys are identical
            }
            //apply gravity
            let r_unit_vec = vec_from_points(actors[i].pos, actors[j].pos).normalize();
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
        //add little bit of dark matter gravity towards origin to avoid "exploding galaxys"
        let origin_vec = vec_from_points(actors[i].pos, Point2::origin());
        actors[i].velocity += origin_vec * G_DARK;

        //calculate new position of this actor
        let delta_pos = actors[i].velocity * dt;
        actors[i].pos += delta_pos;
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
