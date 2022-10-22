use std::f32::consts::PI;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle, ui::FocusPolicy};
use bevy_mod_picking::prelude::{
    backends::raycast::{PickRaycastSource, PickRaycastTarget, RaycastBackend},
    *,
};
use bevy_rapier3d::na::ComplexField;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_framepace::FramepacePlugin) // significantly reduces input lag
        .add_plugins_with(DefaultPickingPlugins::build(RaycastBackend), |group| {
            group.disable::<CustomHighlightingPlugin<ColorMaterial>>()
        })
        .add_plugin(DebugEventsPlugin::default())
        .insert_resource(WindowDescriptor {
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..Default::default()
        })
        .add_startup_system(setup)
        .add_system(drag_squares)
        .add_system(drag_over_squares)
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
        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
                transform: Transform::from_xyz(x as f32 * 200.0, 0.0, 0.0)
                    .with_scale(Vec3::splat(100.)),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                ..Default::default()
            })
            .insert_bundle(PickableBundle::default()) // <- Makes the mesh pickable.
            .insert(PickRaycastTarget::default()) // <- Needed for the raycast backend.
            .insert(SpinMe(0.0));
    }

    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(PickRaycastSource::default()); // <- Sets the camera to use for picking.
}

fn drag_squares(
    // Pointer Events
    mut drag_start: EventReader<PointerDragStart>,
    mut drags: EventReader<PointerDrag>,
    mut drag_end: EventReader<PointerDragEnd>,
    // Inputs
    map: Res<PointerMap>,
    windows: Res<Windows>,
    locations: Query<&PointerLocation>,
    // Outputs
    mut square: Query<(&mut Transform, &mut Handle<ColorMaterial>, &mut FocusPolicy)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for start in drag_start.iter() {
        let (_, mut square_matl, mut policy) = square.get_mut(start.target()).unwrap();
        *square_matl = materials.add(ColorMaterial::from(Color::YELLOW_GREEN));
        *policy = FocusPolicy::Pass;
    }

    for drag in drags.iter() {
        let pointer_entity = map.get_entity(drag.pointer_id()).unwrap();
        let pointer_location = locations.get(pointer_entity).unwrap().location();
        let pointer_position = pointer_location.unwrap().position;
        let window = windows.get_primary().unwrap();
        let window_size = Vec2::new(window.width(), window.height());
        let (mut square_transform, _, _) = square.get_mut(drag.target()).unwrap();
        square_transform.translation = (pointer_position - (window_size / 2.0)).extend(1.0);
    }

    for end in drag_end.iter() {
        let (mut square_transform, mut square_matl, mut policy) =
            square.get_mut(end.target()).unwrap();
        square_transform.translation.z = 0.0;
        *square_matl = materials.add(ColorMaterial::from(Color::WHITE));
        *policy = FocusPolicy::Block;
    }
}

fn drag_over_squares(
    // Pointer Events
    mut drag_enter: EventReader<PointerDragEnter>,
    mut drag_leave: EventReader<PointerDragLeave>,
    // Outputs
    mut square: Query<&mut Handle<ColorMaterial>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for enter in drag_enter.iter() {
        if let Ok(mut square_matl) = square.get_mut(enter.target()) {
            *square_matl = materials.add(ColorMaterial::from(Color::BLUE));
        }
    }
    for leave in drag_leave.iter() {
        if let Ok(mut square_matl) = square.get_mut(leave.target()) {
            *square_matl = materials.add(ColorMaterial::from(Color::WHITE));
        }
    }
}

fn drop_squares(mut drop: EventReader<PointerDrop>, mut square: Query<&mut SpinMe>) {
    for drop in drop.iter() {
        if let Ok(mut spin) = square.get_mut(drop.target()) {
            if spin.0.abs() <= 0.01 {
                spin.0 = 0.5 * PI;
            }
        }
        if let Ok(mut spin) = square.get_mut(drop.event_data().dropped) {
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
