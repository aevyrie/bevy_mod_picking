use bevy::{
    prelude::*,
    render::camera::{RenderTarget, Viewport},
    window::{CreateWindow, PresentMode, WindowId, WindowResized},
};
use bevy_mod_picking::prelude::*;

static SECOND_WINDOW_ID: bevy::render::once_cell::sync::Lazy<WindowId> =
    bevy::render::once_cell::sync::Lazy::new(WindowId::new);
const SECONDARY_EGUI_PASS: &str = "secondary_egui_pass";

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(bevy_framepace::FramepacePlugin) // significantly reduces input lag
        .add_plugin(bevy_egui::EguiPlugin)
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .add_system(make_pickable)
        .add_system(set_camera_viewport);

    let render_app = app.sub_app_mut(bevy::render::RenderApp);
    let mut graph = render_app
        .world
        .get_resource_mut::<bevy::render::render_graph::RenderGraph>()
        .unwrap();

    bevy_egui::setup_pipeline(
        &mut graph,
        bevy_egui::RenderGraphConfig {
            window_id: *SECOND_WINDOW_ID,
            egui_pass: SECONDARY_EGUI_PASS,
        },
    );

    app.run();
}

#[derive(Component)]
struct ViewportCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut create_window_events: EventWriter<CreateWindow>,
) {
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        PickRaycastTarget::default(), // <- Needed for the raycast backend.
    ));

    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        PickRaycastTarget::default(), // <- Needed for the raycast backend.
    ));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, -4.0),
        ..Default::default()
    });

    // main camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        PickRaycastSource::default(), // <- Enable picking for this camera
        ViewportCamera,
    ));

    // sends out a "CreateWindow" event, which will be received by the windowing backend
    create_window_events.send(CreateWindow {
        id: *SECOND_WINDOW_ID,
        descriptor: WindowDescriptor {
            width: 800.,
            height: 600.,
            present_mode: PresentMode::AutoNoVsync,
            title: "Second window".to_string(),
            ..default()
        },
    });

    // second window camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(4.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                target: RenderTarget::Window(*SECOND_WINDOW_ID),
                ..default()
            },
            ..default()
        },
        PickRaycastSource::default(),
    ));
}

fn make_pickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Handle<Mesh>>, Without<PickRaycastTarget>)>,
) {
    for entity in meshes.iter() {
        commands
            .entity(entity)
            .insert((PickableBundle::default(), PickRaycastTarget::default()));
    }
}

fn set_camera_viewport(
    windows: Res<Windows>,
    mut resize_events: EventReader<WindowResized>,
    mut viewport_camera: Query<&mut Camera, With<ViewportCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.iter() {
        if resize_event.id == WindowId::primary() {
            let window = windows.primary();
            let mut left_camera = viewport_camera.single_mut();
            left_camera.viewport = Some(Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: UVec2::new(
                    window.physical_width() / 2,
                    window.physical_height() / 3,
                ),
                ..default()
            });
        }
    }
}
