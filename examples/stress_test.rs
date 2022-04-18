use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use bevy_mod_picking::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "bevy_mod_picking stress test".to_string(),
            width: 800.,
            height: 600.,
            present_mode: PresentMode::Immediate,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction, and Highlighting plugins.
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let half_width: isize = 5;
    let subdivisions: usize = 50;
    let tris_sphere = 20 * subdivisions.pow(2);
    let tris_total = tris_sphere * (half_width as usize * 2).pow(3);
    info!("Total tris: {}, Tris per mesh: {}", tris_total, tris_sphere);

    let mesh_handle = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.2,
        subdivisions,
    }));

    let matl_handle = materials.add(StandardMaterial {
        perceptual_roughness: 0.5,
        metallic: 0.6,
        base_color: Color::hsla(0.0, 0.0, 0.3, 1.0),
        ..Default::default()
    });

    // Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(half_width as f32, half_width as f32, half_width as f32)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());

    // Spawn a cube of spheres.
    for x in -half_width..half_width {
        for y in -half_width..half_width {
            for z in -half_width..half_width {
                commands
                    .spawn_bundle(PbrBundle {
                        mesh: mesh_handle.clone(),
                        material: matl_handle.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            x as f32 + 0.35,
                            y as f32 - 1.0,
                            z as f32,
                        )),
                        ..Default::default()
                    })
                    .insert_bundle(PickableBundle::default());
            }
        }
    }

    // Light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(half_width as f32, half_width as f32, half_width as f32),
        point_light: PointLight {
            intensity: 2500.0,
            ..Default::default()
        },
        ..Default::default()
    });
}
