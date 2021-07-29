use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_mod_picking::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            vsync: false, // Disabled for this demo to remove vsync as a source of input latency
            ..Default::default()
        })
        .insert_resource(bevy::pbr::AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(WindowDescriptor {
            title: "bevy_mod_picking stress test".to_string(),
            width: 800.,
            height: 600.,
            vsync: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(DefaultPickingPlugins)
        .insert_resource(SceneInstance::default())
        .add_startup_system(setup)
        .add_system(scene_update)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut scene_instance: ResMut<SceneInstance>,
) {
    let instance_id =
        scene_spawner.spawn(asset_server.load("models/Sponza/glTF/Sponza.gltf#Scene0"));
    scene_instance.0 = Some(instance_id);

    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(3.0, 2.5, 0.0),
                Vec3::new(0.0, 3.0, 0.0),
                Vec3::Y,
            )),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(3.0, 5.0, 3.0),
        ..Default::default()
    });
}

// Resource to hold the scene `instance_id` until it is loaded
#[derive(Default)]
struct SceneInstance(Option<bevy::scene::InstanceId>);

fn scene_update(
    mut commands: Commands,
    scene_spawner: Res<SceneSpawner>,
    scene_instance: Res<SceneInstance>,
    mut done: Local<bool>,
) {
    if !*done {
        if let Some(instance_id) = scene_instance.0 {
            if let Some(entity_iter) = scene_spawner.iter_instance_entities(instance_id) {
                entity_iter.for_each(|entity| {
                    commands
                        .entity(entity)
                        .insert_bundle(PickableBundle::default());
                });
                *done = true;
            }
        }
    }
}
