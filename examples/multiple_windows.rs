use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    window::{CreateWindow, PresentMode, WindowId},
};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins::build(RaycastBackend))
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .add_system(make_pickable)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut create_window_events: EventWriter<CreateWindow>,
) {
    // plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default()) // <- Makes the mesh pickable.
        .insert(PickRaycastTarget::default()); // <- Needed for the raycast backend.

    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert_bundle(PickableBundle::default()) // <- Makes the mesh pickable.
        .insert(PickRaycastTarget::default()); // <- Needed for the raycast backend.

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // main camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(PickRaycastSource::default());

    let window_id = WindowId::new();

    // sends out a "CreateWindow" event, which will be received by the windowing backend
    create_window_events.send(CreateWindow {
        id: window_id,
        descriptor: WindowDescriptor {
            width: 800.,
            height: 600.,
            present_mode: PresentMode::AutoNoVsync,
            title: "Second window".to_string(),
            ..default()
        },
    });

    // second window camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(4.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                target: RenderTarget::Window(window_id),
                ..default()
            },
            ..default()
        })
        .insert(PickRaycastSource::default());
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
