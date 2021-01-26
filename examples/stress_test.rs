use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin},
    prelude::*,
};
use bevy_mod_picking::*;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            title: "bevy_mod_picking stress test".to_string(),
            width: 800.,
            height: 600.,
            vsync: false,
            ..Default::default()
        })
        //.add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(HighlightablePickingPlugin)
        //.add_plugin(DebugPickingPlugin)
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
                    f32::from(edge_length) * -0.55,
                    f32::from(edge_length) * 0.55,
                    f32::from(edge_length) * 0.45,
                ),
                Vec3::new(
                    f32::from(edge_length) * 0.1,
                    0.0,
                    -f32::from(edge_length) * 0.1,
                ),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .with_bundle(PickingCameraBundle::default());

    let _scenes: Vec<HandleUntyped> = asset_server.load_folder("models").unwrap();
    let mesh_handle = asset_server.get_handle("models/monkey/Monkey.gltf#Mesh0/Primitive0");
    for i in 0..edge_length.pow(3) {
        let f_edge_length = edge_length as f32;
        commands
            .spawn(PbrBundle {
                mesh: mesh_handle.clone(),
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(
                    i as f32 % f_edge_length - f_edge_length / 2.0,
                    (i as f32 / f_edge_length).round() % f_edge_length - f_edge_length / 2.0,
                    (i as f32 / (f_edge_length * f_edge_length)).round() % f_edge_length
                        - f_edge_length / 2.0,
                )) * Transform::from_scale(Vec3::from([0.25, 0.25, 0.25])),
                ..Default::default()
            })
            //.with(PickableMesh::default())
            .with_bundle(PickableBundle::default())
            .with(BoundVol::default());
    }

    commands.spawn(LightBundle {
        transform: Transform::from_translation(Vec3::new(
            0.0,
            f32::from(edge_length),
            f32::from(edge_length),
        )),
        ..Default::default()
    });
}
