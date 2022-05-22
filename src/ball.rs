use bevy::ecs::component::Component;
use bevy::math::Vec2;

use crate::quadtree::QuadRect;

#[derive(Component, Default)]
pub struct Ball {
    current_pos: Vec2,
    prev_pos: Vec2,
    acceleration: Vec2,

    pub radius: f32,

    pub rect: QuadRect,
    id: usize,
}

impl Ball {
    pub fn new(pos: Vec2, r: f32, vel: Vec2) -> Self {
        Ball {
            current_pos: pos,
            prev_pos: -vel,
            radius: r,

            rect: QuadRect {
                pos: Vec2::new(pos.x - r / 2.0, pos.y - r / 2.0),
                dim: Vec2::new(r, r),
            },
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

    pub fn apply_constraints(&mut self, pos: Vec2, dim: Vec2) {
        // Top
        if self.current_pos.y - self.radius < pos.y {
            self.current_pos.y = pos.y + self.radius;
        }

        // Bottom
        if self.current_pos.y + self.radius > pos.y + dim.y {
            self.current_pos.y = pos.y + dim.y - self.radius;
        }

        // Left
        if self.current_pos.x - self.radius < pos.x {
            self.current_pos.x = pos.x + self.radius;
        }

        // Right
        if self.current_pos.x + self.radius > pos.x + dim.x {
            self.current_pos.x = pos.x + dim.x - self.radius;
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

    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    pub fn get_position(&self) -> Vec2 {
        self.current_pos
    }

    pub fn get_prev_position(&self) -> Vec2 {
        self.prev_pos
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
}
