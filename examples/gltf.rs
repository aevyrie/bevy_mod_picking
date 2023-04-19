use bevy::{math::vec4, prelude::*};
use bevy_mod_picking::prelude::*;
use highlight::HighlightKind;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
        )
        .add_startup_system(setup)
        .add_system(make_pickable)
        .add_event::<HelmetClicked>()
        .add_system(HelmetClicked::print_events)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.7, 0.7, 1.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..default()
        },
        RaycastPickCamera::default(), // <- Sets the camera to use for picking.;
    ));
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight { ..default() },
        ..default()
    });
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("models/FlightHelmet/FlightHelmet.gltf#Scene0"),
            ..default()
        },
        // Events that target children of the scene will bubble up to this level and will fire off a
        // `HelmetClicked` event.
        EventListener::<Click>::forward_event::<HelmetClicked>(),
    ));
}

struct HelmetClicked(Entity);
impl<E: IsPointerEvent> ForwardedEvent<E> for HelmetClicked {
    fn from_data(event_data: &ListenedEvent<E>) -> Self {
        // Note that we forward the target, not the listener! The target is the child that the event
        // was targeting, whereas the listener is the parent with the `EventListener` component.
        // This is what allows us to add a listener to the parent scene, yet still know precisely
        // which child entity was clicked on.
        Self(event_data.target)
    }
}
impl HelmetClicked {
    fn print_events(mut click_events: EventReader<HelmetClicked>) {
        for event in click_events.iter() {
            info!("Clicked on: {:?}!", event.0);
        }
    }
}

/// Makes everything in the scene with a mesh pickable
fn make_pickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Handle<Mesh>>, Without<RaycastPickTarget>)>,
) {
    for entity in meshes.iter() {
        commands.entity(entity).insert((
            PickableBundle::default(),
            RaycastPickTarget::default(),
            HIGHLIGHT_TINT.clone(),
        ));
    }
}

const HIGHLIGHT_TINT: Highlight<StandardMaterial> = Highlight {
    hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.5, -0.3, 0.9, 0.8),
        ..matl.to_owned()
    })),
    pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.4, -0.4, 0.8, 0.8),
        ..matl.to_owned()
    })),
    selected: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.4, 0.8, -0.4, 0.0),
        ..matl.to_owned()
    })),
};
