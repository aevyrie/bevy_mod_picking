//! Demonstrates how to spawn and control a virtual pointer, useful for integration testing or
//! something like a gamepad-controlled software pointer.

use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    utils::Uuid,
    window::{PrimaryWindow, WindowRef},
};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(bevy_egui::EguiPlugin) // Nicer pointer debug overlay, useful for this example.
        .add_systems(Startup, setup)
        .add_systems(Update, move_virtual_pointer)
        .run();
}

#[derive(Component)]
pub struct VirtualPointer;

fn move_virtual_pointer(
    time: Res<Time>,
    mut pointer: Query<&mut PointerLocation, With<VirtualPointer>>,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
) {
    let t = time.elapsed_seconds() * 0.5;
    for mut pointer in &mut pointer {
        let w = windows.single().1.width();
        let h = windows.single().1.height();
        pointer.location = Some(pointer::Location {
            target: RenderTarget::Window(WindowRef::Primary)
                .normalize(windows.get_single().ok().map(|w| w.0))
                .unwrap(),
            position: Vec2 {
                x: w * (0.5 + 0.25 * t.sin()),
                y: h * (0.5 + 0.25 * (t * 2.0).sin()),
            }
            .round(),
        });
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a new pointer. This is our "virtual" pointer we can control manually.
    commands.spawn((
        VirtualPointer,
        PointerBundle::new(PointerId::Custom(Uuid::new_v4())),
    ));

    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
    ));

    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
    ));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, -4.0),
        ..Default::default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        RaycastPickCamera::default(), // <- Enable picking for this camera
    ));
}
