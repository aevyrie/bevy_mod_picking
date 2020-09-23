use bevy::prelude::*;
use bevy_mod_picking::*;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_plugin(PickingPlugin)
        .add_plugin(DebugPickingPlugin)
        .add_startup_system(setup.system())
        .add_startup_system(set_highlight_params.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
    // camera
    commands
        .spawn(Camera3dComponents {
            transform: Transform::new(Mat4::face_toward(
                Vec3::new(-3.0, 5.0, 8.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .with(PickingSource::default())
        //plane
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            ..Default::default()
        })
        .with(PickableMesh::default())
        .with(HighlightablePickMesh::new())
        .with(SelectablePickMesh::new())
        // cube
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            ..Default::default()
        })
        .with(PickableMesh::default())
        .with(HighlightablePickMesh::new())
        .with(SelectablePickMesh::new())
        // sphere
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                subdivisions: 4,
                radius: 0.5,
            })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(1.5, 1.5, 1.5)),
            ..Default::default()
        })
        .with(PickableMesh::default())
        .with(HighlightablePickMesh::new())
        .with(SelectablePickMesh::new())
        // light
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}

fn set_highlight_params(mut highlight_params: ResMut<PickHighlightParams>) {
    highlight_params.set_hover_color(Color::rgb(1.0, 0.0, 0.0));
    highlight_params.set_selection_color(Color::rgb(1.0, 0.0, 1.0));
}
