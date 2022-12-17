use bevy::{prelude::*, utils::Uuid, window::WindowId};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(bevy_framepace::FramepacePlugin) // significantly reduces input lag
        .add_startup_system(setup)
        .add_system(move_virtual_pointer)
        .run();
}

#[derive(Component)]
pub struct VirtualPointer;

fn move_virtual_pointer(
    time: Res<Time>,
    mut pointer: Query<&mut PointerLocation, With<VirtualPointer>>,
    windows: ResMut<Windows>,
) {
    for mut pointer in &mut pointer {
        let w = windows.primary().width();
        let h = windows.primary().height();
        pointer.location = Some(pointer::Location {
            target: bevy::render::camera::RenderTarget::Window(WindowId::primary()),
            position: Vec2 {
                x: w * (0.5 + 0.25 * time.elapsed_seconds().sin()),
                y: h * (0.5 + 0.25 * (time.elapsed_seconds() * 2.0).sin()),
            },
        });
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a new pointer. This is our "virtual" pointer we can control manually.
    commands.spawn((
        VirtualPointer,
        PointerBundle::new(PointerId::Custom(Uuid::new_v4())),
    ));

    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        PickRaycastTarget::default(), // <- Needed for the raycast backend.
    ));

    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        PickRaycastTarget::default(), // <- Needed for the raycast backend.
    ));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, -4.0),
        ..Default::default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
            // Uncomment the following lines to try out orthographic projection:
            //
            // projection: bevy::render::camera::Projection::Orthographic(OrthographicProjection {
            //     scale: 0.01,
            //     ..Default::default()
            // }),
            ..Default::default()
        },
        PickRaycastSource::default(), // <- Enable picking for this camera
    ));
}
