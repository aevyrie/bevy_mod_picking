use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_picking::prelude::{
    backends::raycast::{PickRaycastSource, PickRaycastTarget, RaycastBackend},
    *,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins_with(DefaultPickingPlugins::build(RaycastBackend), |group| {
            group.disable::<CustomHighlightingPlugin<ColorMaterial>>()
        })
        .add_plugin(DebugEventsPlugin::default())
        .insert_resource(WindowDescriptor {
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..Default::default()
        })
        .add_startup_system(setup)
        .add_system(drag)
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
            .insert(PickRaycastTarget::default()); // <- Needed for the raycast backend.
    }

    commands
        .spawn_bundle(Camera2dBundle::default())
        .insert(PickRaycastSource::default()); // <- Sets the camera to use for picking.
}

fn drag(
    // Pointer Events
    mut drag_start: EventReader<PointerDragStart>,
    mut drags: EventReader<PointerDrag>,
    mut drag_end: EventReader<PointerDragEnd>,
    // Inputs
    map: Res<PointerMap>,
    windows: Res<Windows>,
    locations: Query<&PointerLocation>,
    // Outputs
    mut square: Query<(&mut Transform, &mut Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for start in drag_start.iter() {
        let (_, mut square_matl) = square.get_mut(start.target()).unwrap();
        *square_matl = materials.add(ColorMaterial::from(Color::YELLOW_GREEN));
    }

    for drag in drags.iter() {
        let pointer_entity = map.get_entity(drag.pointer_id()).unwrap();
        let pointer_loc = locations.get(pointer_entity).unwrap().location();
        let pointer_pos = pointer_loc.unwrap().position;

        let window = windows.get_primary().unwrap();
        let window_size = Vec2::new(window.width(), window.height());

        let (mut square_transform, _) = square.get_mut(drag.target()).unwrap();
        square_transform.translation = (pointer_pos - (window_size / 2.0)).extend(1.0);
    }

    for end in drag_end.iter() {
        let (mut square_transform, mut square_matl) = square.get_mut(end.target()).unwrap();
        square_transform.translation.z = 0.0;
        *square_matl = materials.add(ColorMaterial::from(Color::WHITE));
    }
}
