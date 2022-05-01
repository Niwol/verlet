use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::mesh::PrimitiveTopology,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

pub struct VerletPlugin;

impl Plugin for VerletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_ball).add_system(update_balls);
    }
}

#[derive(Component, Default)]
pub struct Ball {
    x: f32,
    y: f32,
}

#[derive(Default, Component)]
pub struct CustomColor {
    color: Color,
}

fn spawn_ball(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if input.just_pressed(KeyCode::Space) {
        let mut m = Mesh::new(PrimitiveTopology::TriangleList);

        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=20 {
            let di = (i as f32 / 20.0) * 2.0 * 3.1415;

            let x = di.cos();
            let y = di.sin();

            let r = (x * 255.0) as u32;
            let g = (y * 255.0) as u32;
            let b = (((x + y) / 2.0) * 255.0) as u32;
            let c = (r << (8 * 3)) + (g << (8 * 2)) + (b << (8 * 1)) + (0x000000ff);
            // println!("{{ {}, {}, {} }}", r, g, b);

            vertices.push([x, y, 0.0]);
            normals.push([x, y, 0.0]);
            colors.push(c);
        }

        for i in 1..20 {
            indices.push(0);
            indices.push(i);
            indices.push(i + 1);
        }
        let indices = bevy::render::mesh::Indices::U16(indices);

        m.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        m.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        m.insert_attribute(Mesh::ATTRIBUTE_UV_0, None);
        m.set_indices(Some(indices));

        let mesh_bundle = ColorMesh2dBundle {
            mesh: meshes.add(m).into(),
            material: materials.add(ColorMaterial::from(Color::BLUE)),
            ..default()
        };

        commands
            .spawn_bundle(mesh_bundle)
            .insert(Ball { ..default() });
    }
}

fn update_balls(time: Res<Time>, mut query: Query<&mut Transform, With<Ball>>) {
    for mut t in query.iter_mut() {
        t.translation.y -= 50.0 * time.delta_seconds();
    }
}
