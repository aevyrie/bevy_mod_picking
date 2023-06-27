//! This example demonstrates how backends can be mixed and matched, specifically with egui. Here,
//! we are using the egui backend, which is enabled automatically in `DefaultPickingPlugins` when
//! the "egui_backend" feature is enabled. The egui backend will automatically apply a `NoDeselect`
//! component to the egui entity, which allows you to interact with the UI without deselecting
//! anything in the 3d scene.

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, ScrollArea},
    EguiContexts, EguiPlugin,
};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Update, ui_example)
        .add_systems(Startup, setup)
        .run();
}

fn ui_example(mut egui_contexts: EguiContexts) {
    egui::Window::new("Demo").show(egui_contexts.ctx_mut(), |ui| {
        ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
            ui.heading("Note that you can select a 3d object then click on the egui window without that object being deselected!");
        });
    });
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, -4.0),
        ..Default::default()
    });
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        RaycastPickCamera::default(), // <- Enable picking for this camera
    ));
}
