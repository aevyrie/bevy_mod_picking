use std::f32::consts::FRAC_PI_2;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_picking::{
    events::{Bubble, EventListener, PointerEvent},
    prelude::{
        backends::raycast::{PickRaycastCamera, PickRaycastTarget},
        *,
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(bevy_framepace::FramepacePlugin) // significantly reduces input lag
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
            PickRaycastTarget::default(), // <- Needed for the raycast backend.
            EventListener::<DragStart>::callback(make_non_pickable),
            EventListener::<DragEnd>::callback(make_pickable),
            EventListener::<Drop>::callback(spin_target),
        ));
    }

    commands.spawn((Camera2dBundle::default(), PickRaycastCamera::default())); // <- Sets the camera to use for picking.
}

/// When we start dragging, we don't want this entity to prevent picking squares underneath
fn make_non_pickable(
    commands: &mut Commands,
    event: &EventListenerData<DragStart>,
    _: &mut Bubble,
) {
    commands
        .entity(event.target())
        .remove::<PickRaycastTarget>();
}

fn make_pickable(commands: &mut Commands, event: &EventListenerData<DragEnd>, _: &mut Bubble) {
    commands
        .entity(event.target())
        .insert(PickRaycastTarget::default());
}

fn spin_target(commands: &mut Commands, event: &EventListenerData<Drop>, _: &mut Bubble) {
    let dropped = event.event().dropped_entity;
    commands.entity(dropped).insert(SpinTarget(FRAC_PI_2));
    let onto = event.target();
    commands.entity(onto).insert(SpinTarget(-FRAC_PI_2));
}

#[allow(clippy::too_many_arguments)]
fn drag_squares(
    mut drag_events: EventReader<PointerEvent<Drag>>,
    pointers: Res<PointerMap>,
    windows: Query<(Entity, &Window)>,
    images: Res<Assets<Image>>,
    locations: Query<&PointerLocation>,
    // Outputs
    mut square: Query<(Entity, &mut Transform)>,
) {
    // While being dragged, update the position of the square to be under the pointer.
    for dragging in drag_events.iter() {
        let pointer_entity = pointers.get_entity(dragging.pointer_id()).unwrap();
        let pointer_location = locations.get(pointer_entity).unwrap().location().unwrap();
        let pointer_position = pointer_location.position;
        let target = pointer_location
            .target
            .get_render_target_info(&windows, &images)
            .unwrap();
        let target_size = target.physical_size.as_vec2() / target.scale_factor as f32;

        let (_, mut square_transform) = square.get_mut(dragging.target()).unwrap();
        let z = square_transform.translation.z;
        square_transform.translation = (pointer_position - (target_size / 2.0)).extend(z);
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
