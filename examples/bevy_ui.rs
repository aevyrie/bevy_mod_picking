//! This example demonstrates how to use the plugin with bevy_ui.

use bevy::{ecs::system::EntityCommands, prelude::*, ui::FocusPolicy};
use bevy_eventlistener::prelude::*;
use bevy_mod_picking::prelude::*;

const NORMAL: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED: Color = Color::rgb(0.35, 0.75, 0.35);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_3d)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    commands.spawn(Camera2dBundle::default());
    let root = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(500.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                margin: UiRect::horizontal(Val::Auto),
                ..default()
            },
            // *** Important! ***
            //
            // We need to use `FocusPolicy::Pass` here so the root node doesn't block pointer
            // interactions from reaching the 3d objects under the UI. This node, as defined, will
            // stretch from the top to bottom of the screen, take the width of the buttons, but will
            // be invisible. Try commenting out this line or setting it to `Block` to see how
            // behavior changes.
            focus_policy: FocusPolicy::Pass,
            ..default()
        })
        .id();

    commands
        .entity(root)
        .add_button(&font, "Start")
        .add_button(&font, "Settings")
        .add_button(&font, "Quit");
}

/// set up a simple 3D scene
fn setup_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::from_size(5.0))),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        },
        PickableBundle::default(),    // <- Makes the mesh pickable.
        RaycastPickTarget::default(), // <- Needed for the raycast backend.
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
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
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        RaycastPickCamera::default(), // <- Enable picking for this camera
    ));
}

trait NewButton {
    fn add_button(self, font: &Handle<Font>, text: &str) -> Self;
}

impl<'w, 's, 'a> NewButton for EntityCommands<'w, 's, 'a> {
    fn add_button(mut self, font: &Handle<Font>, text: &str) -> Self {
        let child = self
            .commands()
            .spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(42.0),
                        margin: UiRect::top(Val::Percent(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL.into(),
                    focus_policy: FocusPolicy::Block,
                    ..default()
                },
                // Use events to highlight buttons
                On::<Pointer<Over>>::listener_insert(BackgroundColor::from(HOVERED)),
                On::<Pointer<Out>>::listener_insert(BackgroundColor::from(NORMAL)),
                On::<Pointer<Down>>::listener_insert(BackgroundColor::from(PRESSED)),
                On::<Pointer<Up>>::listener_insert(BackgroundColor::from(HOVERED)),
                // Buttons should not deselect other things:
                NoDeselect,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        text,
                        TextStyle {
                            font: font.to_owned(),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ),
                    // If we don't block, the event will be sent twice, because picking the text
                    // will pass through and pick the button container as well.
                    focus_policy: FocusPolicy::Block,
                    ..Default::default()
                });
            })
            .id();
        self.add_child(child);
        self
    }
}
