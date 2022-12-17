use std::f32::consts::PI;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_picking::prelude::{
    backends::raycast::{PickRaycastSource, PickRaycastTarget},
    *,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(bevy_framepace::FramepacePlugin) // significantly reduces input lag
        .add_startup_system(setup)
        .add_system(drag_squares)
        .add_system(drop_squares)
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
            SpinMe(0.0),
        ));
    }

    commands.spawn((Camera2dBundle::default(), PickRaycastSource::default())); // <- Sets the camera to use for picking.
}

#[allow(clippy::too_many_arguments)]
fn drag_squares(
    mut commands: Commands,
    // Pointer Events
    mut drag_start_events: EventReader<PointerDragStart>,
    mut drag_events: EventReader<PointerDrag>,
    mut drag_end_events: EventReader<PointerDragEnd>,
    // Inputs
    pointers: Res<PointerMap>,
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
    locations: Query<&PointerLocation>,
    // Outputs
    mut square: Query<(Entity, &mut Transform)>,
) {
    // When we start dragging a square, we need to change the focus policy so that picking passes
    // through it. Because the square will be locked to the cursor, it will block the pointer and we
    // won't be able to tell what we are dropping it onto unless we do this.
    for drag_start in drag_start_events.iter() {
        let (entity, _) = square.get_mut(drag_start.target()).unwrap();
        commands.entity(entity).remove::<PickRaycastTarget>();
    }

    // While being dragged, update the position of the square to be under the pointer.
    for dragging in drag_events.iter() {
        let pointer_entity = pointers.get_entity(dragging.pointer_id()).unwrap();
        let pointer_location = locations.get(pointer_entity).unwrap().location().unwrap();
        let pointer_position = pointer_location.position;
        let target_size = pointer_location
            .target
            .get_render_target_info(&windows, &images)
            .unwrap()
            .physical_size
            .as_vec2();

        let (_, mut square_transform) = square.get_mut(dragging.target()).unwrap();
        let z = square_transform.translation.z;
        square_transform.translation = (pointer_position - (target_size / 2.0)).extend(z);
    }

    //
    for drag_end in drag_end_events.iter() {
        let (entity, _) = square.get_mut(drag_end.target()).unwrap();
        commands.entity(entity).insert(PickRaycastTarget::default());
    }
}

fn drop_squares(mut drop_events: EventReader<PointerDrop>, mut square_spin: Query<&mut SpinMe>) {
    for dropped in drop_events.iter() {
        if let Ok(mut spin) = square_spin.get_mut(dropped.target()) {
            if spin.0.abs() <= 0.01 {
                spin.0 = 0.5 * PI;
            }
        }
        if let Ok(mut spin) = square_spin.get_mut(dropped.event_data().dropped_entity) {
            if spin.0.abs() <= 0.01 {
                spin.0 = -0.5 * PI;
            }
        }
    }
}

#[derive(Component)]
struct SpinMe(f32);

fn spin(mut square: Query<(&mut SpinMe, &mut Transform)>) {
    for (mut spin, mut transform) in square.iter_mut() {
        transform.rotation = Quat::from_rotation_z(spin.0);
        let delta = -spin.0.clamp(-1.0, 1.0) * 0.05;
        spin.0 += delta;
    }
}
