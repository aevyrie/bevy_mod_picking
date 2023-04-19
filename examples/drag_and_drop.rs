use std::f32::consts::FRAC_PI_2;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_startup_system(setup)
        .add_system(drag_squares)
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
            EventListener::<DragStart>::callback(start_dragging),
            EventListener::<DragEnd>::callback(stop_dragging),
            EventListener::<Drop>::callback(spin_target),
        ));
    }
}

/// Needed to track where on the square the drag started
enum ClickOffset {
    World(Vec2),
    Local(Vec2),
}

#[derive(Component)]
struct FollowPointer {
    camera: Entity,
    offset: ClickOffset,
}

fn start_dragging(commands: &mut Commands, event: &ListenedEvent<DragStart>, _: &mut Bubble) {
    commands
        .entity(event.target)
        .remove::<RaycastPickTarget>() // allow picking squares underneath this one
        .insert(FollowPointer {
            camera: event.inner.hit.camera,
            offset: ClickOffset::World(event.inner.hit.position.unwrap().truncate()),
        });
}

fn stop_dragging(commands: &mut Commands, event: &ListenedEvent<DragEnd>, _: &mut Bubble) {
    commands
        .entity(event.target)
        .insert(RaycastPickTarget::default())
        .remove::<FollowPointer>();
}

fn spin_target(commands: &mut Commands, event: &ListenedEvent<Drop>, _: &mut Bubble) {
    let dropped = event.inner.dropped_entity;
    commands.entity(dropped).insert(SpinTarget(FRAC_PI_2));
    let onto = event.target;
    commands.entity(onto).insert(SpinTarget(-FRAC_PI_2));
}

// While being dragged, update the position of the square to be under the pointer.
fn drag_squares(
    mut drag_events: EventReader<PointerEvent<Drag>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut square: Query<(&mut FollowPointer, &mut Transform)>,
) {
    for dragging in drag_events.iter() {
        let Ok((mut follow, mut square_transform)) = square.get_mut(dragging.target) else {
            continue;
        };
        if let ClickOffset::World(pos) = follow.offset {
            follow.offset = ClickOffset::Local(pos - square_transform.translation.truncate())
        };
        if let ClickOffset::Local(offset) = follow.offset {
            let (camera, cam_transform) = cameras.get(follow.camera).unwrap();
            let pointer_world =
                camera.viewport_to_world_2d(cam_transform, dragging.pointer_location.position);
            square_transform.translation =
                (pointer_world.unwrap() - offset).extend(square_transform.translation.z);
        }
    }
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
