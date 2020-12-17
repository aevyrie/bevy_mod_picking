use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin},
    prelude::*,
};
use bevy_mod_picking::*;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let edge_length: usize = 10;
    let subdivision: usize = 40;
    println!("Tris per mesh: {}", (subdivision + 1).pow(2) * 20);
    println!(
        "Total tris: {}",
        (subdivision + 1).pow(2) * 20 * edge_length.pow(3)
    );

    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(-3.0, 5.0, 8.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .with(PickSource::default());

    for i in 0..edge_length.pow(3) {
        let f_edge_length = edge_length as f32;
        let _a = commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.25,
                    subdivisions: subdivision,
                })),
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(
                    i as f32 % f_edge_length - f_edge_length / 2.0,
                    (i as f32 / f_edge_length).round() % f_edge_length - f_edge_length / 2.0,
                    (i as f32 / (f_edge_length * f_edge_length)).round() % f_edge_length
                        - f_edge_length / 2.0,
                )),
                ..Default::default()
            })
            .with(PickableMesh::default())
            .with(HighlightablePickMesh::default())
            .with(SelectablePickMesh::default());
    }

    commands.spawn(LightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
}
