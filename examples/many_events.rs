//! A test of very large hierarchies for stress testing event listening.

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
        ))
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, setup)
        .run();
}

const WIDTH: isize = 32;
const HEIGHT: isize = 18;
const TREE_DEPTH: usize = 100;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = &meshes.add(Cuboid::default());
    let material = &materials.add(Color::rgb(0.8, 0.7, 0.6));

    for x in -WIDTH..=WIDTH {
        for y in -HEIGHT..=HEIGHT {
            let mut latest = commands
                .spawn((
                    SpatialBundle::default(),
                    On::<Pointer<Click>>::run(move || info!("I've been clicked! ({x}, {y})")),
                ))
                .id();
            for i in 0..=TREE_DEPTH {
                let child = if i == TREE_DEPTH {
                    let transform = Transform::from_xyz(x as f32, y as f32, -WIDTH as f32 * 1.5);
                    spawn_cube(&mut commands, mesh, material, transform)
                } else {
                    commands.spawn(SpatialBundle::default()).id()
                };
                commands.entity(latest).add_child(child);
                latest = child;
            }
        }
    }

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 150.0 * WIDTH as f32,
            ..default()
        },
        ..default()
    });
    commands.spawn((Camera3dBundle::default(),));
}

fn spawn_cube(
    commands: &mut Commands<'_, '_>,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    transform: Transform,
) -> Entity {
    commands
        .spawn((
            PbrBundle {
                mesh: mesh.clone(),
                material: material.clone(),
                transform,
                ..default()
            },
            PickableBundle::default(), // <- Makes the mesh pickable.
        ))
        .id()
}
