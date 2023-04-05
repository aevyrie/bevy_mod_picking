//! Provides sensible defaults for mouse picking inputs.

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
};
use bevy_picking_core::{
    pointer::{InputMove, InputPress, Location, PointerButton, PointerId},
    PointerCoreBundle,
};

/// Spawns the default mouse pointer.
pub fn spawn_mouse_pointer(mut commands: Commands) {
    commands.spawn((
        PointerCoreBundle::new(PointerId::Mouse),
        #[cfg(feature = "selection")]
        bevy_picking_selection::PointerMultiselect::default(),
    ));
}

/// Sends mouse pointer events to be processed by the core plugin
pub fn mouse_pick_events(
    // Input
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut cursor_moves: EventReader<CursorMoved>,
    mut mouse_inputs: EventReader<MouseButtonInput>,
    // Output
    mut pointer_move: EventWriter<InputMove>,
    mut pointer_presses: EventWriter<InputPress>,
) {
    for event in cursor_moves.iter() {
        pointer_move.send(InputMove::new(
            PointerId::Mouse,
            Location {
                target: RenderTarget::Window(WindowRef::Entity(event.window))
                    .normalize(Some(windows.single().0))
                    .unwrap(),
                position: event.position,
            },
        ));
    }

    for input in mouse_inputs.iter() {
        let button = match input.button {
            MouseButton::Left => PointerButton::Primary,
            MouseButton::Right => PointerButton::Secondary,
            MouseButton::Middle => PointerButton::Middle,
            MouseButton::Other(_) => continue,
        };

        match input.state {
            ButtonState::Pressed => {
                pointer_presses.send(InputPress::new_down(PointerId::Mouse, button))
            }
            ButtonState::Released => {
                pointer_presses.send(InputPress::new_up(PointerId::Mouse, button))
            }
        }
    }
}
