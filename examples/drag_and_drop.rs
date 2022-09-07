use bevy::prelude::*;
use bevy_mod_picking::prelude::{
    backends::raycast::{PickRaycastSource, PickRaycastTarget, RaycastPlugin},
    *,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(RaycastPlugin)
        .add_startup_system(setup)
        .add_startup_system(add_cubes)
        .add_system(CubeDrop::handle_events)
        .run();
}

/// Create an event that is triggered when a cube has been dropped on another cube.
struct CubeDrop {
    dropped: Entity,
    target: Entity,
}
impl ForwardedEvent<PointerDrop> for CubeDrop {
    fn from_data(event_data: &PointerEventData<PointerDrop>) -> CubeDrop {
        CubeDrop {
            dropped: event_data.event().dropped,
            target: event_data.target(),
        }
    }
}
impl CubeDrop {
    fn handle_events(mut drop: EventReader<CubeDrop>) {
        for event in drop.iter() {
            info!("{:?} dropped on {:?}", event.dropped, event.target);
        }
    }
}

/// Add pickable cubes to the scene that will forward `PointerDrop` events to our custom
/// `CubeDropEvent`.
fn add_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for i in -1..2 {
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::WHITE.into()),
                transform: Transform::from_xyz(0.0, 2.0 * i as f32, 0.0),
                ..Default::default()
            })
            .insert_bundle(PickableBundle::default())
            .insert(PickRaycastTarget::default())
            .forward_events::<PointerDrop, CubeDrop>();
    }
}

/// Setup not relevant to the example.
fn setup(mut commands: Commands) {
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(PickRaycastSource::default());
}
