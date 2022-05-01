use bevy::prelude::*;

use verlet::VerletPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(VerletPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
