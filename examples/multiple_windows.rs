use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{CreateWindow, PresentMode, WindowId},
};
use bevy_mod_picking::{
    DebugEventsPlugin, DefaultPickingPlugins, PickRaycastSource, PickRaycastTarget, PickableBundle,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction, and Highlighting plugins.
        .add_plugin(DebugEventsPlugin) // <- Adds debug event logging.
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    mut create_window_events: EventWriter<CreateWindow>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default()) // <- Makes the mesh pickable.
        .insert(PickRaycastTarget::default()); // <- Needed for the raycast backend.

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 5.0, 4.0),
        ..default()
    });

    // main camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(PickRaycastSource::default()); // <- Sets the camera to use for picking.;

    let window_id = WindowId::new();

    // sends out a "CreateWindow" event, which will be received by the windowing backend
    create_window_events.send(CreateWindow {
        id: window_id,
        descriptor: WindowDescriptor {
            width: 800.,
            height: 600.,
            present_mode: PresentMode::Immediate,
            title: "Second window".to_string(),
            ..default()
        },
    });

    // second window camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(4.0, 4.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                target: RenderTarget::Window(window_id),
                ..default()
            },
            ..default()
        })
        .insert(PickRaycastSource::default()); // <- Sets the camera to use for picking.;
}
