use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin},
    prelude::*,
    window::WindowMode,
};
use bevy_mod_picking::*;
use rand::prelude::*;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "bevy_mod_picking stress test".to_string(),
            width: 800.,
            height: 600.,
            vsync: false,
            resizable: true,
            mode: WindowMode::Windowed,
            ..Default::default()
        })
        //.add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(DebugPickingPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(PrintDiagnosticsPlugin::default())
        .add_startup_system(setup.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let edge_length: u16 = 18;
    println!("Total tris: {}", 3936 * i32::from(edge_length).pow(3));

    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(
                    -f32::from(edge_length) / 3.0,
                    f32::from(edge_length) / 2.0,
                    f32::from(edge_length) * 0.8,
                ),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .with(PickSource::default());

    let _scenes: Vec<HandleUntyped> = asset_server.load_folder("models/monkey").unwrap();
    let monkey_handle = asset_server.get_handle("models/monkey/Monkey.gltf#Mesh0/Primitive0");
    let mut rng = thread_rng();
    for i in 0..edge_length.pow(3) {
        let f_edge_length = edge_length as f32;
        commands
            .spawn(PbrBundle {
                mesh: monkey_handle.clone(),
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(
                    (i as f32 % f_edge_length - f_edge_length / 2.0) + rng.gen::<f32>() - 0.5,
                    ((i as f32 / f_edge_length).round() % f_edge_length - f_edge_length / 2.0)
                        + rng.gen::<f32>()
                        - 0.5,
                    ((i as f32 / (f_edge_length * f_edge_length)).round() % f_edge_length
                        - f_edge_length / 2.0)
                        + rng.gen::<f32>()
                        - 0.5,
                )) * Transform::from_scale(Vec3::from([0.3, 0.3, 0.3])),
                ..Default::default()
            })
            .with(PickableMesh::default().with_bounding_sphere(monkey_handle.clone()));
    }

    commands.spawn(LightBundle {
        transform: Transform::from_translation(Vec3::new(
            -f32::from(edge_length),
            f32::from(edge_length),
            f32::from(edge_length),
        )),
        ..Default::default()
    });
}
