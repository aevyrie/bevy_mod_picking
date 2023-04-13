use bevy::{math::vec4, prelude::*};
use bevy_mod_picking::prelude::*;
use highlight::HighlightKind;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.2,
        })
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_startup_system(setup)
        .add_system(make_pickable)
        .add_system(HelmetClicked::handle_events)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.7, 0.7, 1.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..default()
        },
        PickRaycastCamera::default(), // <- Sets the camera to use for picking.;
    ));
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight { ..default() },
        ..default()
    });
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"),
            ..default()
        })
        // Check out this neat trick!
        //
        // Because event forwarding uses event bubbling, events that target children of the scene
        // will bubble up to this level and will fire off a `HelmetClicked` event.
        .forward_events::<Click, HelmetClicked>();
}

struct HelmetClicked(Entity);
impl<E: IsPointerEvent> ForwardedEvent<E> for HelmetClicked {
    fn from_data(event_data: &EventListenerData<E>) -> Self {
        // Note that we forward the target, not the listener! The target is the child that the event
        // was originally called on, whereas the listener is the parent entity that was listening
        // for the event that bubbled up from the target. This is what allows us to add a listener
        // to the parent scene, yet still know exactly what child entity was interacted with.
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
        commands.entity(entity).insert((
            PickableBundle::default(),
            PickRaycastTarget::default(),
            HIGHLIGHT_TINT.clone(),
        ));
    }
}

const HIGHLIGHT_TINT: HighlightOverride<StandardMaterial> = HighlightOverride {
    hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.2, -0.2, 0.4, 0.0),
        ..matl.to_owned()
    })),
    pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.3, -0.3, 0.5, 0.0),
        ..matl.to_owned()
    })),
    selected: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.3, 0.2, -0.3, 0.0),
        ..matl.to_owned()
    })),
};
