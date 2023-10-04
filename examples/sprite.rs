//! Demonstrates how to use the bevy_sprite picking backend. This backend simply tests the bounds of
//! a sprite.

use bevy::{prelude::*, sprite::Anchor};
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

fn move_sprite(
    time: Res<Time>,
    mut sprite: Query<&mut Transform, (Without<Sprite>, With<Children>)>,
) {
    let t = time.elapsed_seconds() * 0.1;
    for mut transform in &mut sprite {
        let new = Vec2 {
            x: 50.0 * t.sin(),
            y: 50.0 * (t * 2.0).sin(),
        };
        transform.translation.x = new.x;
        transform.translation.y = new.y;
    }
}

/// Set up a scene that tests all sprite anchor types.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let len = 128.0;
    let sprite_size = Some(Vec2::splat(len / 2.0));

    commands
        .spawn(SpatialBundle::default())
        .with_children(|commands| {
            for (anchor_index, anchor) in [
                Anchor::TopLeft,
                Anchor::TopCenter,
                Anchor::TopRight,
                Anchor::CenterLeft,
                Anchor::Center,
                Anchor::CenterRight,
                Anchor::BottomLeft,
                Anchor::BottomCenter,
                Anchor::BottomRight,
                Anchor::Custom(Vec2::new(0.5, 0.5)),
            ]
            .iter()
            .enumerate()
            {
                let i = (anchor_index % 3) as f32;
                let j = (anchor_index / 3) as f32;

                // spawn black square behind sprite to show anchor point
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        custom_size: sprite_size,
                        color: Color::BLACK,
                        ..default()
                    },
                    transform: Transform::from_xyz(i * len - len, j * len - len, -1.0),
                    ..default()
                });

                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        custom_size: sprite_size,
                        color: Color::RED,
                        anchor: anchor.to_owned(),
                        ..default()
                    },
                    texture: asset_server.load("images/boovy.png"),
                    // 3x3 grid of anchor examples by changing transform
                    transform: Transform::from_xyz(i * len - len, j * len - len, 0.0)
                        .with_scale(Vec3::splat(1.0 + (i - 1.0) * 0.2))
                        .with_rotation(Quat::from_rotation_z((j - 1.0) * 0.2)),
                    ..default()
                });
            }
        });
}
