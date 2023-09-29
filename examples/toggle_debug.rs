//! Shows how to toggle debug logging and the pointer debug overlay at runtime
//!
//! This is all essentially identical to bevy_ui, except the buttons
//! are configured to send custom events, and new small systems which
//! react to the button clicks. `cycle_logging()` shows how to change
//! the State which controls debug log verbosity.
//!
//! Note that the visual overlay next to the pointer is enabled with
//! debug logging on, and disabled when it is off.

use bevy::app::AppExit;
use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_eventlistener::prelude::*;
use bevy_mod_picking::prelude::*;

// See bevy_eventlistener. In particular, look at the event_listeners.rs example.
#[derive(Clone, Event)]
struct CycleLogging(Entity);

impl From<ListenerInput<Pointer<Click>>> for CycleLogging {
    fn from(event: ListenerInput<Pointer<Click>>) -> Self {
        CycleLogging(event.target) // you could use this to choose between different buttons
    }
}

// change log verbosity by cycling through the DebugPickingMode state
fn cycle_logging(
    logging_state: Res<State<debug::DebugPickingMode>>,
    mut logging_next_state: ResMut<NextState<debug::DebugPickingMode>>,
) {
    match logging_state.get() {
        debug::DebugPickingMode::Normal => {
            info!("Changing state from Normal to Noisy.");
            logging_next_state.set(debug::DebugPickingMode::Noisy);
        }
        debug::DebugPickingMode::Noisy => {
            info!("Changing state from Noisy to Disabled.");
            logging_next_state.set(debug::DebugPickingMode::Disabled);
        }
        debug::DebugPickingMode::Disabled => {
            info!("Changing state from Disabled to Normal.");
            logging_next_state.set(debug::DebugPickingMode::Normal);
        }
    }
}

// basically same as above, but does something different.
#[derive(Clone, Event)]
struct Shutdown;

impl From<ListenerInput<Pointer<Click>>> for Shutdown {
    fn from(_event: ListenerInput<Pointer<Click>>) -> Self {
        Shutdown
    }
}

fn shutdown(mut eventwriter_exit: EventWriter<bevy::app::AppExit>) {
    eventwriter_exit.send(AppExit);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(low_latency_window_plugin()))
        .add_plugins(DefaultPickingPlugins)
        // If you don't add the events, code will build but crash at runtime
        .add_event::<CycleLogging>()
        .add_event::<Shutdown>()
        .add_systems(Startup, (setup, setup_3d))
        .add_systems(Update, update_button_colors)
        // add our button-event response systems, set to only run when the
        // respective events are triggered.
        .add_systems(Update, cycle_logging.run_if(on_event::<CycleLogging>()))
        .add_systems(Update, shutdown.run_if(on_event::<Shutdown>()))
        .run();
}

// Everything below this line is identical to what's in bevy_ui, except 
// the event listener is passed to .add_button along with the text to display.
//----------------------------------------------------------------------------

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

fn setup(mut commands: Commands) {
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
        ))
        .id();

    commands
        .entity(root)
        .add_button(
            "Cycle Logging State",
            On::<Pointer<Click>>::send_event::<CycleLogging>(),
        )
        .add_button("Quit", On::<Pointer<Click>>::send_event::<Shutdown>());
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
        PickableBundle::default(), // <- Makes the mesh pickable.
    ));
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        },
        PickableBundle::default(), // <- Makes the mesh pickable.
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
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            order: 1,
            ..default()
        },
        ..default()
    },));
}

trait NewButton {
    fn add_button(self, text: &str, on_click_action: On<Pointer<Click>>) -> Self;
}

impl<'w, 's, 'a> NewButton for EntityCommands<'w, 's, 'a> {
    fn add_button(mut self, text: &str, on_click_action: On<Pointer<Click>>) -> Self {
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
                on_click_action,
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
                        ..Default::default()
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
