//! Demonstrates how to use the bevy_sprite picking backend. This backend simply tests the bounds of
//! a sprite.
//!
//! This also renders a 3d view in the background, to demonstrate and test that camera order is
//! respected across different backends, in this case the sprite and 3d raycasting backends.

use bevy::{prelude::*, sprite::Anchor};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(low_latency_window_plugin()),
            DefaultPickingPlugins,
        ))
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, (setup, setup_3d, setup_atlas))
        .add_systems(Update, (move_sprite, animate_sprite))
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
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        ..default()
    });

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

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn setup_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle = asset_server.load("images/gabe-idle-run.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(24.0, 24.0), 7, 1, None, None);
    let texture_atlas_layout_handle = texture_atlas_layouts.add(layout);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6 };
    commands.spawn((
        SpriteSheetBundle {
            texture: texture_handle,
            atlas: TextureAtlas {
                layout: texture_atlas_layout_handle,
                index: animation_indices.first,
            },
            transform: Transform::from_xyz(300.0, 0.0, 0.0).with_scale(Vec3::splat(6.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn setup_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(bevy_render::mesh::PlaneMeshBuilder {
                half_size: Vec2::splat(2.5),
                ..default()
            }),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            ..default()
        },
        PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        PickableBundle::default(), // Optional: adds selection, highlighting, and helper components.
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, -4.0),
        ..default()
    });
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));
}
