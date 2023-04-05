use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use highlight::HighlightKind;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(bevy_framepace::FramepacePlugin) // significantly reduces input lag
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let tinted_highlight = HighlightOverride::<StandardMaterial> {
        hovered: Some(HighlightKind::new_dynamic(|i| {
            let [r, g, b, a] = i.base_color.as_rgba_f32();
            StandardMaterial {
                base_color: Color::rgba(r - 0.2, g - 0.2, b + 0.4, a),
                ..i.to_owned()
            }
        })),
        pressed: Some(HighlightKind::new_dynamic(|i| {
            let [r, g, b, a] = i.base_color.as_rgba_f32();
            StandardMaterial {
                base_color: Color::rgba(r - 0.3, g - 0.3, b + 0.5, a),
                ..i.to_owned()
            }
        })),
        selected: Some(HighlightKind::new_dynamic(|i| {
            let [r, g, b, a] = i.base_color.as_rgba_f32();
            StandardMaterial {
                base_color: Color::rgba(r - 0.3, g + 0.3, b - 0.3, a),
                ..i.to_owned()
            }
        })),
    };

    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("images/bavy.png")),
                ..Default::default()
            }),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        PickRaycastTarget::default(), // <- Needed for the raycast backend.
        tinted_highlight.clone(),
    ));

    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("images/bavy.png")),
                ..Default::default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        PickRaycastTarget::default(), // <- Needed for the raycast backend.
        tinted_highlight.clone(),
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
