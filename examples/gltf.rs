use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.2,
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(RaycastPlugin)
        .add_startup_system(setup)
        .add_system(make_pickable)
        .add_system(HelmetClicked::handle_events)
        .run();
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
        // Check out this neat trick!
        //
        // Because event forwarding can rely on event bubbling, events that target children of the
        // scene will bubble up to this level and will fire off a `HelmetClicked` event.
        .forward_events::<PointerClick, HelmetClicked>();
}

struct HelmetClicked(Entity);
impl<E: IsPointerEvent> ForwardedEvent<E> for HelmetClicked {
    fn from_data(event_data: &PointerEventData<E>) -> Self {
        // Note that we forward the target, not the entity! The target is the child that the event
        // was originally called on, whereas the listener is the parent entity that was listening
        // for the event that bubbled up from the target.
        Self(event_data.target())
    }
}
impl HelmetClicked {
    /// Handle our custom forwarded event.
    fn handle_events(mut click_events: EventReader<HelmetClicked>) {
        for event in click_events.iter() {
            info!("Hello {:?}!", event.0);
        }
    }
}

/// Makes everything in the scene with a mesh pickable
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
