use bevy::prelude::*;
use bevy_mod_picking::{
    output::{EventData, EventFrom, EventListenerCommands, IsPointerEvent, PointerClick},
    DefaultPickingPlugins, PickRaycastSource, PickRaycastTarget, PickableBundle,
};

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_startup_system(setup)
        .add_system(make_pickable)
        .add_system(handle_events)
        .run();
}

struct GreetMe(Entity);
impl EventFrom for GreetMe {
    fn new(event_data: &mut EventData<impl IsPointerEvent>) -> Self {
        Self(event_data.target())
    }
}

fn handle_events(mut greets: EventReader<GreetMe>) {
    for event in greets.iter() {
        info!("Hello {:?}!", event.0);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.7, 0.7, 1.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..default()
        })
        .insert(PickRaycastSource::default()); // <- Sets the camera to use for picking.;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight { ..default() },
        ..default()
    });
    commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"),
            ..default()
        })
        .forward_events::<PointerClick, GreetMe>();
}

fn make_pickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Handle<Mesh>>, Without<PickRaycastTarget>)>,
) {
    for entity in meshes.iter() {
        commands
            .entity(entity)
            .insert_bundle(PickableBundle::default())
            .insert(PickRaycastTarget::default());
    }
}
