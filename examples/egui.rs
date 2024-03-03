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
        .add_plugins((
            DefaultPlugins.set(low_latency_window_plugin()),
            DefaultPickingPlugins,
            EguiPlugin,
        ))
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, setup)
        .add_systems(Update, ui_example)
        .run();
}

fn ui_example(mut egui_contexts: EguiContexts, mut number: Local<f32>) {
    egui::SidePanel::left("Left").show(egui_contexts.ctx_mut(), |ui| {
        ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.heading("Note that while a slider is being dragged, the panel is being resized, or the scrollbar is being moved, items in the 3d scene cannot be picked even if the mouse is over them.");
                for _ in 0..100 {
                    ui.add(egui::Slider::new(&mut *number, 0.0..=100.0));
                }
            })
    });
    egui::Window::new("Demo").show(egui_contexts.ctx_mut(), |ui| {
        ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
            ui.heading("Note that you can select a 3d object then click on this egui window without that object being deselected!");
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
            mesh: meshes.add(bevy_render::mesh::PlaneMeshBuilder {
                half_size: Vec2::splat(2.5),
                ..default()
            }),
            material: materials.add(Color::WHITE),
            ..default()
        },
        PickableBundle::default(),
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        PickableBundle::default(),
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, -4.0),
        ..default()
    });
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));
}
