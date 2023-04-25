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
            OnPointer::<DragStart>::run_callback(start_dragging),
            OnPointer::<DragEnd>::run_callback(stop_dragging),
            OnPointer::<Drag>::run_callback(drag_squares),
            OnPointer::<Drop>::run_callback(spin_target),
        ));
    }
}

/// Used to track where the square was clicked, so the square doesn't jump and center itself on the
/// pointer when you start dragging.
#[derive(Component)]
struct DragOffset {
    offset: Vec2,
    camera: Entity,
}

fn start_dragging(
    In(event): In<ListenedEvent<DragStart>>,
    mut commands: Commands,
    mut square: Query<&Transform>,
) -> Bubble {
    let square_transform = square.get_mut(event.target).unwrap();
    commands
        .entity(event.target)
        .remove::<RaycastPickTarget>() // Allow pointer hit tests to pass through while dragging
        .insert(DragOffset {
            offset: (event.hit.position.unwrap() - square_transform.translation).truncate(),
            camera: event.hit.camera,
        });
    Bubble::Up
}

// While being dragged, update the position of the square to be under the pointer.
fn drag_squares(
    In(event): In<ListenedEvent<Drag>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut square: Query<(&DragOffset, &mut Transform)>,
) -> Bubble {
    let (drag, mut square_transform) = square.get_mut(event.target).unwrap();
    let (camera, cam_transform) = cameras.get(drag.camera).unwrap();
    let pointer_world = camera.viewport_to_world_2d(cam_transform, event.pointer_location.position);
    square_transform.translation =
        (pointer_world.unwrap() - drag.offset).extend(square_transform.translation.z);
    Bubble::Up
}

fn stop_dragging(In(event): In<ListenedEvent<DragEnd>>, mut commands: Commands) -> Bubble {
    commands
        .entity(event.target)
        .insert(RaycastPickTarget::default());
    Bubble::Up
}

fn spin_target(In(event): In<ListenedEvent<Drop>>, mut commands: Commands) -> Bubble {
    let dropped = event.dropped_entity;
    commands.entity(dropped).insert(SpinTarget(FRAC_PI_2));
    let onto = event.target;
    commands.entity(onto).insert(SpinTarget(-FRAC_PI_2));
    Bubble::Up
}

#[derive(Component)]
struct SpinTarget(f32);

fn spin(mut square: Query<(&mut SpinTarget, &mut Transform)>) {
    for (mut spin, mut transform) in square.iter_mut() {
        transform.rotation = Quat::from_rotation_z(spin.0);
        let delta = -spin.0.clamp(-1.0, 1.0) * 0.05;
        spin.0 += delta;
    }
}
