//! Demonstrates how to use the bevy_sprite picking backend.
//!
//! You must enable the `backend_sprite` or `all` features.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins::start().with_backend(SpriteBackend))
        .add_plugin(bevy_framepace::FramepacePlugin) // significantly reduces input lag
        .add_startup_system(setup)
        .add_system(move_sprite)
        .run();
}

fn move_sprite(time: Res<Time>, mut sprite: Query<&mut Transform, With<Sprite>>) {
    let mut transform = sprite.single_mut();
    let new = Vec2 {
        x: 200.0 * time.elapsed_seconds().sin(),
        y: 200.0 * (time.elapsed_seconds() * 2.0).sin(),
    };
    transform.translation.x = new.x;
    transform.translation.y = new.y;
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load("images/bavy.png"),
        ..default()
    });
}
