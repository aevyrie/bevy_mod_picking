use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_mod_picking::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "bevy_mod_picking stress test".to_string(),
            width: 800.,
            height: 600.,
            vsync: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(HighlightablePickingPlugin)
        //.add_plugin(DebugCursorPickingPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Spawn 32,768 monkeys
    let half_width: isize = 4;
    let subdivisions: usize = 45;

    // Suzanne has 3936 tris.
    let tris_sphere = 20 * subdivisions.pow(2);
    let tris_total = tris_sphere * (half_width as usize * 2).pow(3);

    info!("Total tris: {}, Tris per mesh: {}", tris_total, tris_sphere);

    let mesh_handle = meshes.add(
        shape::Icosphere {
            radius: 0.2,
            subdivisions,
        }
        .into(),
    );
    //let mesh_handle = asset_server.get_handle("models/monkey/Monkey.gltf#Mesh0/Primitive0");

    let matl_handle = materials.add(StandardMaterial {
        perceptual_roughness: 0.5,
        metallic: 0.6,
        base_color: Color::hsla(0.0, 0.0, 0.3, 1.0),
        ..Default::default()
    });

    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::splat(half_width as f32 * 1.1),
                Vec3::ZERO,
                Vec3::Y,
            )),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());

    let _scenes: Vec<HandleUntyped> = asset_server.load_folder("models").unwrap();
    for x in -half_width..half_width {
        for y in -half_width..half_width {
            for z in -half_width..half_width {
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: mesh_handle.clone(),
                        material: matl_handle.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            x as f32, y as f32, z as f32,
                        )),
                        ..Default::default()
                    })
                    .insert_bundle(PickableBundle::default());
            }
        }
    }

    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_matrix(Mat4::face_toward(
            Vec3::splat(half_width as f32 * 1.1),
            Vec3::ZERO,
            Vec3::Y,
        )),
        point_light: PointLight {
            intensity: 2000.0,
            ..Default::default()
        },
        ..Default::default()
    });
}
