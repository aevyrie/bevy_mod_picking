use bevy::{prelude::*, render::camera::Camera};
use bevy_mod_picking::*;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(DebugPickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_startup_system(setup.system())
        .add_startup_system(set_highlight_params.system())
        .add_system(oscillation_system.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
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
        .with(PickSource::default())
        //plane
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            ..Default::default()
        })
        .with(PickableMesh::default())
        .with(InteractableMesh::default())
        .with(HighlightablePickMesh::default())
        .with(SelectablePickMesh::default())
        // cube
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            ..Default::default()
        })
        .with(PickableMesh::default())
        .with(InteractableMesh::default())
        .with(HighlightablePickMesh::default())
        .with(SelectablePickMesh::default())
        // sphere
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                subdivisions: 20,
                radius: 0.5,
            })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(1.5, 1.5, 1.5)),
            ..Default::default()
        })
        .with(PickableMesh::default())
        .with(InteractableMesh::default())
        .with(HighlightablePickMesh::default())
        .with(SelectablePickMesh::default())
        // light
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}

fn set_highlight_params(mut highlight_params: ResMut<PickHighlightParams>) {
    highlight_params.set_hover_color(Color::rgb(1.0, 0.0, 0.0));
    highlight_params.set_selection_color(Color::rgb(1.0, 0.0, 1.0));
}

fn oscillation_system(time: Res<Time>, mut query: Query<&mut Transform, With<Camera>>) {
    for mut transform in query.iter_mut() {
        transform.translation.y = 5.0 + time.seconds_since_startup().sin() as f32;
    }
}
