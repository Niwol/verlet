use bevy::ecs::component::Component;
use bevy::math::Vec2;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Component)]
pub struct RcBall {
    rcball: Arc<Mutex<Ball>>,
}

#[derive(Component, Default)]
pub struct Ball {
    current_pos: Vec2,
    prev_pos: Vec2,
    acceleration: Vec2,
    radius: f32,
}

impl Ball {
    pub fn new(pos: Vec2, r: f32, vel: Vec2) -> Self {
        Ball {
            current_pos: pos,
            prev_pos: -vel,
            radius: r,
            ..Default::default()
        }
    }

    pub fn accelerate(&mut self, acc: Vec2) {
        self.acceleration += acc;
    }

    pub fn update_position(&mut self, dt: f32) {
        let vel = self.current_pos - self.prev_pos;
        self.prev_pos = self.current_pos;
        self.current_pos = self.current_pos + vel + self.acceleration * dt * dt;
        self.acceleration = Vec2::new(0.0, 0.0);
    }

    pub fn apply_constraints(&mut self, _w: f32, _h: f32) {
        let center = Vec2::new(0.0, 0.0);
        let radius = 300.0;

        let to_obj = self.current_pos - center;
        let dist = to_obj.length();

        if dist > radius - self.radius {
            let n = to_obj.normalize();
            self.current_pos = center + n * (radius - self.radius);
        }
    }

    pub fn solve_colision(&mut self, other: &mut Self) {
        let colision_axes = self.current_pos - other.current_pos;
        let dist = colision_axes.length();
        let min_dist = self.radius + other.radius;

        if dist < min_dist {
            let n = colision_axes.normalize();
            let delta = min_dist - dist;
            self.current_pos += 0.5 * delta * n;
            other.current_pos -= 0.5 * delta * n;
        }
    }

    pub fn get_position(&self) -> Vec2 {
        self.current_pos
    }
}

pub struct BallVector<'a> {
    vec: Vec<&'a RcBall>,
}

impl<'a> BallVector<'a> {
    pub fn new() -> Self {
        BallVector { vec: Vec::new() }
    }

    pub fn push(&mut self, ball: &'a RcBall) {
        self.vec.push(ball);
    }
}
