use bevy::prelude::*;
use bevy_mod_picking::*;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_startup_system(setup.system())
        .add_system(event_example.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world
    commands
        // camera
        .spawn(Camera3dComponents {
            transform: Transform::from_matrix(Mat4::face_toward(
                Vec3::new(-3.0, 5.0, 8.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        })
        .with(PickSource::default())
        //plane
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            ..Default::default()
        })
        .with(PickableMesh::default())
        .with(InteractableMesh::default())
        // cube
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
            ..Default::default()
        })
        .with(PickableMesh::default())
        .with(InteractableMesh::default())
        // light
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}

fn event_example(query: Query<(&InteractableMesh, Entity)>) {
    for (interactable, entity) in &mut query.iter() {
        let hover_event_text = match interactable.hover_event(&Group::default()).unwrap() {
            HoverEvents::None => "",
            HoverEvents::JustEntered => "Mouse Entered",
            HoverEvents::JustExited => "Mouse Exited",
        };
        let hovered_text = interactable.hovered(&Group::default()).unwrap().to_string();
        let mouse_down_event_text = match interactable
            .mouse_down_event(&Group::default(), MouseButton::Left)
            .unwrap()
        {
            MouseDownEvents::None => "",
            MouseDownEvents::MouseJustPressed => "Mouse Pressed",
            MouseDownEvents::MouseJustReleased => "Mouse Released",
        };
        if hover_event_text.is_empty() && mouse_down_event_text.is_empty() {
            continue;
        }
        println!(
            "ENTITY: {:?}, HOVER: {}, HOVER EVENT: {}, CLICK_EVENT: {}",
            entity, hovered_text, hover_event_text, mouse_down_event_text
        );
    }
}
