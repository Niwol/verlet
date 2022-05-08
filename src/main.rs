use bevy::prelude::*;

use verlet::VerletPlugin;

use std::cell::RefCell;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Verlet".to_string(),
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(VerletPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

struct Point {
    x: f32,
    y: f32,
}

fn _f() {
    let mut points = Vec::new();
    for i in 0..10 {
        points.push(RefCell::new(Point {
            x: i as f32,
            y: (i + 5) as f32,
        }));
    }

    for (i, rp1) in points.iter().enumerate() {
        for (j, rp2) in points.iter().enumerate() {
            if i != j {
                let mut p1 = rp1.try_borrow_mut().unwrap();
                let mut p2 = rp2.try_borrow_mut().unwrap();

                p1.x += 2.0;
                p2.y += 3.0;

                println!(
                    "p1: {{ {}, {} }}    p2: {{ {}, {} }}",
                    p1.x, p1.y, p2.x, p2.y
                );
            }
        }
    }
}
