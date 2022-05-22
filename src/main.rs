use bevy::prelude::*;

use verlet::VerletPlugin;

mod quadtree;
use quadtree::QuadTree;

// use std::cell::RefCell;

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

    // let mut vec = Vec::new();

    // let mut qt = QuadTree::new(3, Vec2::new(0.0, 0.0), Vec2::new(200.0, 200.0));

    // vec.push(qt.add(1, Vec2::new(10.0, 10.0), Vec2::new(10.0, 10.0)));
    // vec.push(qt.add(2, Vec2::new(60.0, 10.0), Vec2::new(10.0, 10.0)));
    // vec.push(qt.add(3, Vec2::new(10.0, 60.0), Vec2::new(10.0, 10.0)));
    // vec.push(qt.add(4, Vec2::new(60.0, 60.0), Vec2::new(10.0, 10.0)));
    // vec.push(qt.add(5, Vec2::new(45.0, 60.0), Vec2::new(10.0, 10.0)));
    // vec.push(qt.add(6, Vec2::new(120.0, 60.0), Vec2::new(10.0, 10.0)));
    // vec.push(qt.add(7, Vec2::new(120.0, 95.0), Vec2::new(10.0, 10.0)));

    // qt.remove(vec.remove(1));

    // qt.relocate_contained(
    //     vec[1],
    //     Vec2::new(60.0, 10.0),
    //     Vec2::new(10.0, 10.0),
    //     Vec2::new(120.0, 120.0),
    //     Vec2::new(10.0, 10.0),
    // );

    // qt.remove_with_rect_contained(vec.remove(1), Vec2::new(60.0, 10.0), Vec2::new(10.0, 10.0));
    // qt.remove_with_rect_contained(vec.remove(2), Vec2::new(59.0, 59.0), Vec2::new(12.0, 12.0));

    // println!("QuadTree:");
    // println!("{}", qt);
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

// struct Point {
//     x: f32,
//     y: f32,
// }

// fn f() {
//     let mut points = Vec::new();
//     for i in 0..10 {
//         points.push(RefCell::new(Point {
//             x: i as f32,
//             y: (i + 5) as f32,
//         }));
//     }

//     for (i, rp1) in points.iter().enumerate() {
//         for (j, rp2) in points.iter().enumerate() {
//             if i != j {
//                 let mut p1 = rp1.borrow_mut();
//                 let mut p2 = rp2.borrow_mut();

//                 p1.x += 2.0;
//                 p2.y += 3.0;

//                 println!(
//                     "p1: {{ {}, {} }}    p2: {{ {}, {} }}",
//                     p1.x, p1.y, p2.x, p2.y
//                 );
//             }
//         }
//     }
// }
