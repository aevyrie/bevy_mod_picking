use bevy::prelude::*;
use bevy_mod_picking::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        // PickingPlugin provides core picking systems and must be registered first
        .add_plugin(PickingPlugin)
        // InteractablePickingPlugin adds mouse events and selection
        .add_plugin(InteractablePickingPlugin)
        // HighlightablePickingPlugin adds hover, click, and selection highlighting
        .add_plugin(HighlightablePickingPlugin)
        .add_startup_system(setup.system())
        .add_system_to_stage(stage::POST_UPDATE, print_events.system())
        .run();
}

pub fn print_events(mut events: EventReader<PickingEvent>) {
    for event in events.iter() {
        info!("{:?}", event);
    }
}

fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
    commands
        // camera
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(-3.0, 5.0, 8.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .with_bundle(PickingCameraBundle::default())
        //plane
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            ..Default::default()
        })
        .with_bundle(PickableBundle::default())
        // cube
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            ..Default::default()
        })
        .with_bundle(PickableBundle::default())
        // light
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}
