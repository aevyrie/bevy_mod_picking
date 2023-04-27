use std::f32::consts::FRAC_PI_2;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_startup_system(setup)
        .add_system(spin)
        .run();
}

/// Set up a simple 2D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn camera
    commands.spawn((Camera2dBundle::default(), RaycastPickCamera::default()));
    // Spawn squares
    for x in -2..=2 {
        let z = 0.5 + x as f32 * 0.1;
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                transform: Transform::from_xyz(x as f32 * 200.0, 0.0, z)
                    .with_scale(Vec3::splat(100.)),
                material: materials.add(ColorMaterial::from(Color::hsl(0.0, 1.0, z))),
                ..Default::default()
            },
            PickableBundle::default(),    // <- Makes the mesh pickable.
            RaycastPickTarget::default(), // <- Needed for the raycast backend.
            OnPointer::<DragStart>::add_target_commands(|_, target_commands| {
                target_commands.remove::<RaycastPickTarget>(); // Disable picking the dragged entity
            }),
            OnPointer::<DragEnd>::add_target_commands(|_, target_commands| {
                target_commands.insert(RaycastPickTarget::default()); // Restore removed component
            }),
            OnPointer::<Drag>::run_callback(drag_squares),
            OnPointer::<Drop>::add_commands(|event, commands| {
                commands.entity(event.dropped).insert(Spin(FRAC_PI_2)); // Spin dropped entity
                commands.entity(event.target).insert(Spin(-FRAC_PI_2)); // Spin dropped-on entity
            }),
        ));
    }
}

/// Update the position of a square while it is being dragged. By using event data instead of winit
/// events, this will work for any pointing hardware (mouse/touch/pen/etc).
fn drag_squares(In(drag): In<ListenedEvent<Drag>>, mut square: Query<&mut Transform>) -> Bubble {
    let mut square_transform = square.get_mut(drag.target).unwrap();
    square_transform.translation += drag.delta.extend(0.0);
    Bubble::Up
}

#[derive(Component)]
struct Spin(f32);

fn spin(mut square: Query<(&mut Spin, &mut Transform)>) {
    for (mut spin, mut transform) in square.iter_mut() {
        transform.rotation = Quat::from_rotation_z(spin.0);
        let delta = -spin.0.clamp(-1.0, 1.0) * 0.05;
        spin.0 += delta;
    }
}
