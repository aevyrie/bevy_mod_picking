//! This example demonstrates how to use the plugin with bevy_ui.

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_mod_picking::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, (setup_3d, setup_ui).chain())
        .add_systems(Update, (update_button_colors, set_camera_viewports))
        .insert_resource(UiScale(1.5))
        .insert_resource(DebugPickingMode::Normal)
        .run();
}

/// Use the [`PickingInteraction`] state of each button to update its color.
fn update_button_colors(
    mut buttons: Query<(Option<&PickingInteraction>, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, mut button_color) in &mut buttons {
        *button_color = match interaction {
            Some(PickingInteraction::Pressed) => Color::rgb(0.35, 0.75, 0.35),
            Some(PickingInteraction::Hovered) => Color::rgb(0.25, 0.25, 0.25),
            Some(PickingInteraction::None) | None => Color::rgb(0.15, 0.15, 0.15),
        }
        .into();
    }
}

fn setup_ui(mut commands: Commands, camera: Query<Entity, With<RightCamera>>) {
    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(500.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    margin: UiRect::horizontal(Val::Auto),
                    ..default()
                },
                ..default()
            },
            // *** Important! ***
            //
            // We need to use `Pickable::IGNORE` here so the root node doesn't block pointer
            // interactions from reaching the 3d objects under the UI.
            //
            // This node, as defined, will stretch from the top to bottom of the screen, take the
            // width of the buttons, but will be invisible. Try commenting out this line or changing
            // it to see what happens.
            Pickable::IGNORE,
            TargetCamera(camera.single()),
        ))
        .id();

    commands
        .entity(root)
        .add_button("Start")
        .add_button("Settings")
        .add_button("Quit");
}

/// set up a simple 3D scene
fn setup_3d(
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
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            ..default()
        },
        PickableBundle::default(), // <- Makes the mesh pickable.
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        PickableBundle::default(), // <- Makes the mesh pickable.
    ));
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, -4.0),
        ..default()
    });
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 20.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        LeftCamera,
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 10., 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                // don't clear on the second camera because the first camera already cleared the
                // window
                clear_color: ClearColorConfig::None,
                // Renders the right camera after the left camera, which has a default priority
                // of 0
                order: 1,
                ..default()
            },
            ..default()
        },
        RightCamera,
    ));
}

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<bevy::window::WindowResized>,
    mut left_camera: Query<&mut Camera, (With<LeftCamera>, Without<RightCamera>)>,
    mut right_camera: Query<&mut Camera, With<RightCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes so then
    // each camera always takes up half the screen. A resize_event is sent when the window is first
    // created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let mut left_camera = left_camera.single_mut();
        left_camera.viewport = Some(bevy::render::camera::Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(
                window.resolution.physical_width() / 2,
                window.resolution.physical_height(),
            ),
            ..default()
        });

        let mut right_camera = right_camera.single_mut();
        right_camera.viewport = Some(bevy::render::camera::Viewport {
            physical_position: UVec2::new(window.resolution.physical_width() / 2, 0),
            physical_size: UVec2::new(
                window.resolution.physical_width() / 2,
                window.resolution.physical_height(),
            ),
            ..default()
        });
    }
}

trait NewButton {
    fn add_button(self, text: &str) -> Self;
}

impl<'a> NewButton for EntityCommands<'a> {
    fn add_button(mut self, text: &str) -> Self {
        let text_string = text.to_string();
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
                    ..default()
                },
                // Add an onclick
                On::<Pointer<Click>>::run(move || info!("Button {text_string} pressed!")),
                // Buttons should not deselect other things:
                NoDeselect,
            ))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle {
                        text: Text::from_section(
                            text,
                            TextStyle {
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ),
                        ..default()
                    },
                    // Text should not be involved in pick interactions.
                    Pickable::IGNORE,
                ));
            })
            .id();
        self.add_child(child);
        self
    }
}
