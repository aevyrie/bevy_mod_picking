//! Provides sensible defaults for mouse picking inputs.

use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
    render::camera::RenderTarget,
    utils::HashMap,
    window::{PrimaryWindow, WindowRef},
};
use bevy_picking_core::{
    pointer::{InputMove, InputPress, Location, PointerButton, PointerId},
    PointerCoreBundle,
};

use crate::InputPluginSettings;

/// Map buttons from Bevy's mouse system to pointer buttons. Access through [`InputPluginSettings`].
/// Note that the values (PointerButton) _must_ be unique, as pressed/released status is tracked
/// at the higher level, and will get confused with multiple mouse buttons mapped to the same
/// pointer button.
#[derive(Debug)]
pub struct MouseButtonMapping(HashMap<MouseButton, Option<PointerButton>>);

impl Default for MouseButtonMapping {
    fn default() -> Self {
        Self(HashMap::from([
            (MouseButton::Left, Some(PointerButton::Primary)),
            (MouseButton::Right, Some(PointerButton::Secondary)),
            (MouseButton::Middle, Some(PointerButton::Middle)),
        ]))
    }
}

impl MouseButtonMapping {
    /// Get the PointerButton mapped to a given MouseButton, if any. There are two levels of
    /// `<Option>` to allow you to distinguish between unconfigured / unknown buttons and
    /// buttons which are mapped to None (which you might do if you don't want to use them
    /// for picking).
    pub fn lookup(&self, mouse_button: MouseButton) -> Option<&Option<PointerButton>> {
        self.0.get(&mouse_button)
    }

    /// Set a mouse-to-pointer mapping.
    pub fn set_mapping(
        &mut self,
        mouse_button: MouseButton,
        pointer_button: Option<PointerButton>,
    ) {
        self.0.insert(mouse_button.clone(), pointer_button.clone());
        match pointer_button {
            Some(p) => debug!(
                "Mouse button {:?} set to Pointer button {:?}",
                mouse_button, p
            ),
            None => debug!("Mouse button {:?} set to no Pointer button", mouse_button),
        };
    }
}

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
    mut cursor_last: Local<Vec2>,
    mut mouse_inputs: EventReader<MouseButtonInput>,
    // Output
    mut pointer_move: EventWriter<InputMove>,
    mut pointer_presses: EventWriter<InputPress>,
    input_plugin_settings: Res<InputPluginSettings>,
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
            event.position - *cursor_last,
        ));
        *cursor_last = event.position;
    }

    for input in mouse_inputs.iter() {
        // map bevy mouse buttons (left, right, middle, other) to primary, secondary, middle
        let button = match input_plugin_settings
            .mouse_button_mapping
            .lookup(input.button)
        {
            Some(Some(mouse_button)) => mouse_button,
            Some(None) => {
                debug!(
                    "Button {:?} from disabled mouse button {:?}",
                    input.state, input.button
                );
                continue;
            }
            None => {
                debug!(
                    "Button {:?} from unconfigured mouse button {:?}",
                    input.state, input.button
                );
                continue;
            }
        };

        match input.state {
            ButtonState::Pressed => {
                pointer_presses.send(InputPress::new_down(PointerId::Mouse, *button))
            }
            ButtonState::Released => {
                pointer_presses.send(InputPress::new_up(PointerId::Mouse, *button))
            }
        }
    }
}
