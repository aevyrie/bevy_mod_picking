use bevy::prelude::*;
use bevy_mod_picking::*;

fn main(){
    App::build()
    .add_resource(Msaa { samples: 4 })
    .add_default_plugins()
    .add_plugin(PickingPlugin)
    .add_startup_system(setup.system())
    .add_system(interactable_demo.system())
    .run();
}

fn setup(mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,)
{
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
        .with(PickSource::default())
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
        .with(InteractableMesh::default())
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
        .with(InteractableMesh::default())
        // light
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        });
}

fn interactable_demo(mut imesh_entities: Query<&InteractableMesh>){
    for imesh in imesh_entities.iter().iter(){

        if imesh.mouse_hover {
            //println!("Hovering!");
        }

        if imesh.mouse_entered {
            println!("Mouse Entered");
        }

        if imesh.mouse_exited {
            println!("Mouse Exited");
        }

        match imesh.mouse_down(MouseButton::Left) {
            Some(v) => println!("Left Mouse Button is Down"),
            None => ()
        }

        match imesh.mouse_just_pressed(MouseButton::Left) {
            Some(v) => println!("Left Mouse just Clicked"),
            None => ()
        }

        match imesh.mouse_just_released(MouseButton::Left){
            Some(v) => println!("Left Mouse just Released"),
            None => ()
        }
    }
}