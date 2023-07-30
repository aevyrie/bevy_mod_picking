//! If you are using the provided highlighting plugin, this example demonstrates how you can define
//! dynamic tints that run a closure to determine the color of a highlight.

use bevy::{math::vec4, prelude::*};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(low_latency_window_plugin()),
            DefaultPickingPlugins,
        ))
        .add_systems(Startup, setup)
        .run();
}

// We can use a dynamic highlight that builds a material based on the entity's base material. This
// allows us to "tint" a material by leaving all other properties - like the texture - unchanged,
// and only modifying the base color. The highlighting plugin handles all the work of caching and
// updating these materials when the base material changes, and swapping it out during pointer
// events.
//
// Note that this works for *any* type of asset, not just bevy's built in materials.
const HIGHLIGHT_TINT: Highlight<StandardMaterial> = Highlight {
    hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.2, -0.2, 0.4, 0.0),
        ..matl.to_owned()
    })),
    pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.3, -0.3, 0.5, 0.0),
        ..matl.to_owned()
    })),
    selected: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.3, 0.2, -0.3, 0.0),
        ..matl.to_owned()
    })),
};

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("images/boovy.png")),
                ..Default::default()
            }),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
        HIGHLIGHT_TINT,               // Override the global highlighting settings for this mesh
    ));

    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(asset_server.load("images/boovy.png")),
                ..Default::default()
            }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
        HIGHLIGHT_TINT,               // Override the global highlighting settings for this mesh
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
            ..Default::default()
        },
        RaycastPickCamera::default(), // <- Enable picking for this camera
    ));
}
