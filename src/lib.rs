mod ball;

use ball::Ball;
use ball::BallVector;
use ball::RcBall;
use bevy::{prelude::*, render::mesh::PrimitiveTopology};
use rand::Rng;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct VerletPlugin;

struct TotalBalls(u64);

impl Plugin for VerletPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TotalBalls(0))
            .insert_resource(BallVector::new())
            .add_system(spawn_ball)
            .add_system(remove_ball)
            .add_system(update_balls);
    }
}

fn spawn_ball(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut total_balls: ResMut<TotalBalls>,
    mut ball_vector: ResMut<&'static BallVector>,
) {
    if !input.pressed(KeyCode::Space) && !input.just_pressed(KeyCode::X) {
        return;
    }

    let r = rand::thread_rng().gen_range(5.0..15.0);
    let vel = Vec2::new(
        rand::thread_rng().gen_range(-2.0..=2.0),
        rand::thread_rng().gen_range(0.0..=1.0),
    );

    let color = Color::rgb(
        rand::thread_rng().gen_range(0.0..=1.0),
        rand::thread_rng().gen_range(0.0..=1.0),
        rand::thread_rng().gen_range(0.0..=1.0),
    );

    let mut m = Mesh::new(PrimitiveTopology::TriangleList);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Vertices
    for i in 0..=20 {
        let di = (i as f32 / 20.0) * 2.0 * 3.1415;

        let x = di.cos() * r;
        let y = di.sin() * r;

        positions.push([x, y, 0.0]);
        normals.push([x, y, 0.0]);
        uvs.push([x, y]);
    }

    // Indices
    for i in 1..20 {
        indices.push(0);
        indices.push(i);
        indices.push(i + 1);
    }
    let indices = bevy::render::mesh::Indices::U32(indices);

    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    m.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    m.set_indices(Some(indices));

    let mesh_bundle = ColorMesh2dBundle {
        mesh: meshes.add(m).into(),
        material: materials.add(ColorMaterial::from(color)),
        ..default()
    };

    let ball = Ball::new(Vec2::new(0.0, 0.0), r, vel);
    let rcball = Arc::new(Mutex::new(ball));
    let comp = RcBall {
        rcball: Arc::clone(&rcball),
    };

    let my_ball = RcBall {
        rcball: Arc::clone(&rcball),
    };

    ball_vector.push(&my_ball);

    commands.spawn_bundle(mesh_bundle).insert(comp);

    total_balls.0 += 1;
    println!("Total balls: {}", total_balls.0);
}

fn remove_ball(mut commands: Commands, query: Query<(Entity, &Ball)>) {
    for (e, b) in query.iter() {
        if b.get_position().y < -400.0 {
            commands.entity(e).despawn();
        }
    }
}

fn update_balls(
    windows: Res<Windows>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Ball)>,
) {
    let window = windows.get_primary().unwrap();
    let dt = time.delta_seconds();

    let sub_steps = 8;
    let sub_dt = dt / sub_steps as f32;

    for _ in 0..sub_steps {
        apply_gravity(&mut query);
        update_positions(sub_dt, &mut query);
        apply_constraints(window, &mut query);
        solve_colision(&mut query);
    }

    for (mut t, b) in query.iter_mut() {
        t.translation.x = b.get_position().x;
        t.translation.y = b.get_position().y;
    }
}

fn apply_gravity(query: &mut Query<(&mut Transform, &mut Ball)>) {
    let g = Vec2::new(0.0, -1000.0);

    for (_, mut b) in query.iter_mut() {
        b.accelerate(g);
    }
}

fn apply_constraints(window: &Window, query: &mut Query<(&mut Transform, &mut Ball)>) {
    for (_, mut b) in query.iter_mut() {
        b.apply_constraints(window.width(), window.height());
    }
}

fn solve_colision(query: &mut Query<(&mut Transform, &mut Ball)>) {
    let mut iterator = query.iter_combinations_mut();

    while let Some([(_, mut b1), (_, mut b2)]) = iterator.fetch_next() {
        b1.solve_colision(&mut b2);
    }
}

fn update_positions(dt: f32, query: &mut Query<(&mut Transform, &mut Ball)>) {
    for (_, mut b) in query.iter_mut() {
        b.update_position(dt);
    }
}
