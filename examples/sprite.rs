//! Demonstrates how to use the bevy_sprite picking backend. This backend simply tests the bounds of
//! a sprite.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(low_latency_window_plugin()),
            DefaultPickingPlugins,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, move_sprite)
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
        texture: asset_server.load("images/boovy.png"),
        ..default()
    });
}
