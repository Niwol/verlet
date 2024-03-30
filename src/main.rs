use bevy::{
    prelude::*,
    render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use rand::{self, Rng};

use acceleration_structures::quadtree::Quadtree;
use acceleration_structures::rect;

const GRAVITY: f32 = -1000.0;

#[rustfmt::skip]
const ARENA: Rect = Rect {
    min: Vec2 { x: -900.0, y: -500.0 },
    max: Vec2 { x:  900.0, y:  500.0 },
};

const QUADTREE_OFFSET: f32 = 50.0;

#[derive(Resource)]
struct QuadtreeRes(Quadtree<Entity>);

#[derive(Component)]
struct QuadtreeMesh;

#[derive(Component, Default)]
struct VerletObject {
    position_current: Vec2,
    position_old: Vec2,
    acceleration: Vec2,
}

#[derive(Component)]
struct Ball {
    radius: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .insert_resource(QuadtreeRes(Quadtree::new(
            rect::Rect::new(
                ARENA.min.x - QUADTREE_OFFSET,
                ARENA.min.y - QUADTREE_OFFSET,
                ARENA.width() + QUADTREE_OFFSET * 2.0,
                ARENA.height() + QUADTREE_OFFSET * 2.0,
            ),
            5,
        )))
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_ball)
        .add_systems(Update, (update_physics, update_transforms).chain())
        .add_systems(Last, update_quadtree_mesh)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut quad_mesh = Mesh::new(
        PrimitiveTopology::LineList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    #[rustfmt::skip]
    let vertices = vec![
        // Top
        [ARENA.min.x, ARENA.min.y, 0.0],
        [ARENA.min.x + ARENA.width(), ARENA.min.y, 0.0],

        // Left
        [ARENA.min.x, ARENA.min.y, 0.0],
        [ARENA.min.x, ARENA.min.y + ARENA.height(), 0.0],

        // Right
        [ARENA.min.x + ARENA.width(), ARENA.min.y, 0.0],
        [ARENA.min.x + ARENA.width(), ARENA.min.y + ARENA.height(), 0.0],

        // Bottom
        [ARENA.min.x, ARENA.min.y + ARENA.height(), 0.0],
        [ARENA.min.x + ARENA.width(), ARENA.min.y + ARENA.height(), 0.0],
    ];

    quad_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    commands
        .spawn(ColorMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(quad_mesh)),
            material: materials.add(Color::rgb(0.0, 1.0, 0.0)),
            ..Default::default()
        })
        .insert(QuadtreeMesh);
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut quadtree: ResMut<QuadtreeRes>,
    input: Res<ButtonInput<KeyCode>>,
    balls: Query<Entity, With<Transform>>,
) {
    if !input.just_pressed(KeyCode::KeyN) && !input.pressed(KeyCode::Space) {
        return;
    }
    let mut rng = rand::thread_rng();

    let mut num_balls = balls.iter().count() as f32;
    let depth = num_balls / 10000.0;

    while num_balls > 360.0 {
        num_balls -= 360.0;
    }

    let radius = rng.gen::<f32>() * 10.0 + 5.0;
    let ball = Ball { radius };
    let mesh = Mesh2dHandle(meshes.add(Circle {
        radius: ball.radius,
    }));
    let material = materials.add(Color::hsl(num_balls, 1.0, 0.5));

    let ball_entity = commands
        .spawn(MaterialMesh2dBundle {
            mesh,
            material,
            transform: Transform::from_xyz(0.0, 0.0, depth),
            ..Default::default()
        })
        .insert(ball)
        .insert(VerletObject {
            position_old: Vec2::new(rng.gen(), rng.gen()),
            ..Default::default()
        })
        .id();

    quadtree.0.insert(
        ball_entity,
        rect::Rect::new_centered(0.0, 0.0, radius / 2.0, radius / 2.0),
    );
}

fn update_physics(
    time: Res<Time>,
    mut quadtree: ResMut<QuadtreeRes>,
    mut balls: Query<(&mut VerletObject, &Ball)>,
) {
    let dt = time.delta().as_secs_f32();

    let num_substeps = 5;

    let sub_dt = dt / num_substeps as f32;
    for _ in 0..num_substeps {
        apply_gravity(&mut balls);
        update_position(sub_dt, &mut balls);
        update_quadtree(&mut quadtree, &balls);
        solve_colisions(&mut balls, &quadtree);
        solve_constraints(&mut balls);
    }
}

fn apply_gravity(verlet_objects: &mut Query<(&mut VerletObject, &Ball)>) {
    for (mut verlet_object, _) in verlet_objects.iter_mut() {
        verlet_object.acceleration.y += GRAVITY;
    }
}

fn update_position(dt: f32, balls: &mut Query<(&mut VerletObject, &Ball)>) {
    for (mut verlet_object, _) in balls.iter_mut() {
        let velocity = verlet_object.position_current - verlet_object.position_old;
        let acc = verlet_object.acceleration;

        verlet_object.position_old = verlet_object.position_current;
        verlet_object.position_current += velocity + acc * dt * dt;
        verlet_object.acceleration = Vec2::ZERO;
    }
}

fn solve_constraints(balls: &mut Query<(&mut VerletObject, &Ball)>) {
    for (mut verlet_object, ball) in balls.iter_mut() {
        let r = ball.radius;

        if verlet_object.position_current.x - r < ARENA.min.x {
            verlet_object.position_current.x = ARENA.min.x + r;
        }

        if verlet_object.position_current.x + r > ARENA.min.x + ARENA.width() {
            verlet_object.position_current.x = ARENA.min.x + ARENA.width() - r;
        }

        if verlet_object.position_current.y - r < ARENA.min.y {
            verlet_object.position_current.y = ARENA.min.y + r;
        }

        if verlet_object.position_current.y + r > ARENA.min.y + ARENA.height() {
            verlet_object.position_current.y = ARENA.min.y + ARENA.height() - r;
        }
    }
}

fn solve_colisions(balls: &mut Query<(&mut VerletObject, &Ball)>, quadtree: &ResMut<QuadtreeRes>) {
    let mut new_positions = Vec::new();
    for entry in quadtree.0.entries() {
        let entity = entry.value();
        let (verlet_object, ball) = balls.get(*entity).unwrap();
        let mut new_position = verlet_object.position_current;

        for other_entity in quadtree.0.get_overlapped(entry.region()) {
            if other_entity == entity {
                continue;
            }

            let (other_verlet_object, other_ball) = balls.get(*other_entity).unwrap();

            let dist = other_verlet_object.position_current - verlet_object.position_current;
            if dist.length() < ball.radius + other_ball.radius {
                let overlap = ball.radius + other_ball.radius - dist.length();

                let n = dist.normalize();
                new_position -= n * (overlap / 2.0);
            }
        }

        new_positions.push((*entity, new_position));
    }

    for (entity, new_position) in new_positions {
        let (mut vo, _) = balls.get_mut(entity).unwrap();
        vo.position_current = new_position;
    }

    //    let mut ball_iter = balls.iter_combinations_mut();
    //    while let Some([(mut vo1, b1), (mut vo2, b2)]) = ball_iter.fetch_next() {
    //        let dist = vo2.position_current - vo1.position_current;
    //        if dist.length() < b1.radius + b2.radius {
    //            let overlap = b1.radius + b2.radius - dist.length();
    //
    //            let n = dist.normalize();
    //            vo1.position_current -= n * (overlap / 2.0);
    //            vo2.position_current += n * (overlap / 2.0);
    //        }
    //    }
}

fn update_transforms(mut balls: Query<(&mut Transform, &VerletObject)>) {
    for (mut transform, verlet_object) in balls.iter_mut() {
        transform.translation = Vec3::new(
            verlet_object.position_current.x,
            verlet_object.position_current.y,
            transform.translation.z,
        );
    }
}

fn update_quadtree_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    quad_mesh: Query<&Mesh2dHandle, With<QuadtreeMesh>>,
    mut quadtree: ResMut<QuadtreeRes>,
    verlet_objects: Query<(&VerletObject, &Ball)>,
) {
    let quad_mesh_handle = quad_mesh.single();
    let quad_mesh = meshes.get_mut(quad_mesh_handle.0.id()).unwrap();

    #[rustfmt::skip]
    let mut vertices = vec![
        // Top
        [ARENA.min.x, ARENA.min.y, 0.0],
        [ARENA.min.x + ARENA.width(), ARENA.min.y, 0.0],

        // Left
        [ARENA.min.x, ARENA.min.y, 0.0],
        [ARENA.min.x, ARENA.min.y + ARENA.height(), 0.0],

        // Right
        [ARENA.min.x + ARENA.width(), ARENA.min.y, 0.0],
        [ARENA.min.x + ARENA.width(), ARENA.min.y + ARENA.height(), 0.0],

        // Bottom
        [ARENA.min.x, ARENA.min.y + ARENA.height(), 0.0],
        [ARENA.min.x + ARENA.width(), ARENA.min.y + ARENA.height(), 0.0],
    ];

    for node in quadtree.0.nodes() {
        if node.is_leaf() {
            continue;
        }

        let region = node.region();
        let l1x1 = region.x + region.w / 2.0;
        let l1y1 = region.y;
        let l1x2 = region.x + region.w / 2.0;
        let l1y2 = region.y + region.h;

        let l2x1 = region.x;
        let l2y1 = region.y + region.h / 2.0;
        let l2x2 = region.x + region.w;
        let l2y2 = region.y + region.h / 2.0;

        vertices.extend_from_slice(&[
            [l1x1, l1y1, 0.0],
            [l1x2, l1y2, 0.0],
            [l2x1, l2y1, 0.0],
            [l2x2, l2y2, 0.0],
        ])
    }

    quad_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
}

fn update_quadtree(
    quadtree: &mut ResMut<QuadtreeRes>,
    verlet_objects: &Query<(&mut VerletObject, &Ball)>,
) {
    for mut entry in quadtree.0.entries_mut() {
        let entity = entry.value();
        let (verlet_object, ball) = verlet_objects.get(*entity).unwrap();
        let new_region = rect::Rect::new_centered(
            verlet_object.position_current.x,
            verlet_object.position_current.y,
            ball.radius * 2.0,
            ball.radius * 2.0,
        );

        entry.move_entry(new_region);
    }
}
