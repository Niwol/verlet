mod ball;
mod quadtree;

use ball::Ball;
use bevy::{prelude::*, render::mesh::PrimitiveTopology};
use quadtree::{QuadRect, QuadTree};
use rand::Rng;

pub struct VerletPlugin;

struct TotalBalls(u64);

#[derive(Component)]
struct QuadMesh {
    handle: Handle<Mesh>,
}

impl Plugin for VerletPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TotalBalls(0))
            .insert_resource(QuadTree::<Entity>::new(
                3,
                QuadRect {
                    pos: Vec2::new(-500.0, -400.0),
                    dim: Vec2::new(1000.0, 800.0),
                },
            ))
            .add_startup_system(setup)
            .add_system(spawn_ball)
            // .add_system(remove_ball)
            .add_system(update_balls)
            .add_system(update_quadtree.after(update_balls));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    qt: Res<QuadTree<Entity>>,
) {
    let mut m = Mesh::new(PrimitiveTopology::LineList);

    let verts = qt.get_vertices();
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    let normals = qt.get_vertices();
    m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    let uvs = qt
        .get_vertices()
        .into_iter()
        .map(|[x, y, _]| [x, y])
        .collect::<Vec<[f32; 2]>>();
    m.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    let quad_mesh = QuadMesh {
        handle: meshes.add(m),
    };

    let mesh_bundle = ColorMesh2dBundle {
        mesh: quad_mesh.handle.clone().into(),
        material: materials.add(ColorMaterial::from(Color::rgb(0.2, 1.0, 0.2))),
        ..default()
    };

    commands.spawn_bundle(mesh_bundle).insert(quad_mesh);
}

fn spawn_ball(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut total_balls: ResMut<TotalBalls>,
    mut qt: ResMut<QuadTree<Entity>>,
) {
    if !input.pressed(KeyCode::Space) && !input.just_pressed(KeyCode::X) {
        return;
    }

    let r = rand::thread_rng().gen_range(2.0..10.0);
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

    let mut ball = Ball::new(Vec2::new(0.0, 0.0), r, vel);

    let entity = commands.spawn_bundle(mesh_bundle).id();

    let id = qt.add(entity, ball.rect);

    ball.set_id(id);

    commands.entity(entity).insert(ball);

    total_balls.0 += 1;
    println!("Total balls: {}", total_balls.0);
}

fn _remove_ball(mut commands: Commands, query: Query<(Entity, &Ball)>) {
    for (e, b) in query.iter() {
        if b.get_position().y < -400.0 {
            commands.entity(e).despawn();
        }
    }
}

fn update_balls(
    windows: Res<Windows>,
    time: Res<Time>,
    qt: Res<QuadTree<Entity>>,
    mut query: Query<(Entity, &mut Transform, &mut Ball)>,
) {
    let window = windows.get_primary().unwrap();
    let dt = time.delta_seconds();

    let sub_steps = 10;
    let sub_dt = dt / sub_steps as f32;

    for _ in 0..sub_steps {
        apply_gravity(&mut query);
        update_positions(sub_dt, &mut query);
        apply_constraints(window, &mut query);
        solve_colision(qt, &mut query);
    }

    for (_, mut t, b) in query.iter_mut() {
        t.translation.x = b.get_position().x;
        t.translation.y = b.get_position().y;
    }
}

fn apply_gravity(query: &mut Query<(Entity, &mut Transform, &mut Ball)>) {
    let g = Vec2::new(0.0, -1000.0);

    for (_, _, mut b) in query.iter_mut() {
        b.accelerate(g);
    }
}

fn apply_constraints(window: &Window, query: &mut Query<(Entity, &mut Transform, &mut Ball)>) {
    for (_, _, mut b) in query.iter_mut() {
        b.apply_constraints(Vec2::new(-500.0, -400.0), Vec2::new(1000.0, 800.0));
    }
}

fn solve_colision(
    qt: Res<QuadTree<Entity>>,
    query: &mut Query<(Entity, &mut Transform, &mut Ball)>,
) {
    // let mut iterator = query.iter_combinations_mut();

    // while let Some([(_, _, mut b1), (_, _, mut b2)]) = iterator.fetch_next() {
    //     b1.solve_colision(&mut b2);
    // }

    for (_, _, b) in query.iter_mut() {
        let others = qt.search_overlaped(b.rect);

        for e in others {
            match query.get_mut(*e) {
                Ok((_, _, mut b2)) => b.solve_colision(&mut b2),
                Err(_) => todo!(),
            }
        }
    }
}

fn update_positions(dt: f32, query: &mut Query<(Entity, &mut Transform, &mut Ball)>) {
    for (_, _, mut b) in query.iter_mut() {
        b.update_position(dt);
    }
}

fn update_quadtree(
    mut qt: ResMut<QuadTree<Entity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&mut Ball>,
    mesh_query: Query<&QuadMesh>,
) {
    for mut b in query.iter_mut() {
        let r = b.radius;

        let pos = b.get_position();
        let dim = Vec2::new(r, r);

        let new_rect = QuadRect {
            pos: Vec2::new(pos.x - r / 2.0, pos.y - r / 2.0),
            dim,
        };

        qt.relocate_contained(b.get_id(), b.rect, new_rect);

        b.rect = new_rect;
        // qt.relocate(b.get_id(), b.get_position(), dim);
    }

    for quad_mesh in mesh_query.iter() {
        match meshes.get_mut(quad_mesh.handle.clone()) {
            Some(m) => {
                let verts = qt.get_vertices();
                m.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);

                let normals = qt.get_vertices();
                m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

                let uvs = qt
                    .get_vertices()
                    .into_iter()
                    .map(|[x, y, _]| [x, y])
                    .collect::<Vec<[f32; 2]>>();
                m.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            }
            None => panic!("AAAAAAAAAAHHHHHHHHHHHHHHHHHHHh"),
        }
    }
}
