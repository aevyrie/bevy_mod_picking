use bevy::prelude::*;
use bevy_mod_picking::*;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_resource(WindowDescriptor {
            vsync: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(DebugPickingPlugin)
        .add_startup_system(setup.system())
        .add_system_to_stage(POST_PICKING, print_events.system())
        .run();
}

fn print_events(query: Query<(&PickableMesh, &InteractableMesh, Entity)>) {
    for (pickable, interactable, entity) in &mut query.iter() {
        let mouse_down_event = interactable.mouse_down_event(MouseButton::Left).unwrap();
        let hover_event = interactable.hover_event();
        // Only print updates if at least one event has occured.
        if hover_event.is_none() && mouse_down_event.is_none() {
            continue;
        }
        let distance = if let Some(intersection) = pickable.intersection() {
            intersection.distance().to_string()
        } else {
            String::from("None")
        };
        println!(
            "ENTITY: {:?}, DIST: {:.4}, EVENT: {:?}, LMB: {:?}",
            entity, distance, hover_event, mouse_down_event
        );
    }
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
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            ..Default::default()
        })
        .with_bundle(PickableBundle::default())
        // sphere
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                subdivisions: 20,
                radius: 0.5,
            })),
            transform: Transform::from_translation(Vec3::new(1.5, 1.5, 1.5)),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            ..Default::default()
        })
        .with_bundle(PickableBundle::default())
        // light
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}
