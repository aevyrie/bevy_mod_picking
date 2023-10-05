//! Provides sensible defaults for mouse picking inputs.

use bevy_ecs::prelude::*;
use bevy_input::{mouse::MouseButtonInput, prelude::*, ButtonState};
use bevy_math::Vec2;
use bevy_reflect::Reflect;
use bevy_render::camera::RenderTarget;
use bevy_utils::{tracing::debug, HashMap};
use bevy_window::{CursorMoved, PrimaryWindow, Window, WindowRef};

use bevy_picking_core::{
    pointer::{InputMove, InputPress, Location, PointerButton, PointerId},
    PointerCoreBundle,
};

/// Map buttons from Bevy's mouse system to pointer buttons.
///
/// Note that the values (PointerButton) _must_ be unique, as pressed/released status is tracked
/// at the higher level, and will get confused with multiple mouse buttons mapped to the same
/// pointer button. Therefore, when you assign a pointer button to a mouse button with [`set_mapping`],
/// any existing mappings to that pointer button will be cleared. For example, if you start
/// from the default and map [`MouseButton::Left`] to [`PointerButton::Secondary`],
/// [`MouseButton::Right`] will be set to `None`. (You can then call `set_mapping` a second time
/// to set `MouseButton::Right` to [`PointerButton::Primary`].)
#[derive(Resource, Debug, Reflect)]
pub struct MouseButtonSettings {
    mapping: HashMap<MouseButton, Option<PointerButton>>,
}

impl Default for MouseButtonSettings {
    /// The default is as one might expect:
    ///
    /// * Left -> Primary
    /// * Right -> Secondary
    /// * Middle -> Middle
    ///
    fn default() -> Self {
        let mut mouse_button_mapping = Self {
            mapping: HashMap::new(),
        };
        mouse_button_mapping.set_mapping(MouseButton::Left, Some(PointerButton::Primary));
        mouse_button_mapping.set_mapping(MouseButton::Right, Some(PointerButton::Secondary));
        mouse_button_mapping.set_mapping(MouseButton::Middle, Some(PointerButton::Middle));
        mouse_button_mapping
    }
}

impl MouseButtonSettings {
    /// Get the PointerButton mapped to a given MouseButton, if any. There are two levels of
    /// `<Option>` to allow you to distinguish between unconfigured / unknown buttons and
    /// buttons which are mapped to None (which you might do if you don't want to use them
    /// for picking).
    pub fn get_mapping(&self, mouse_button: MouseButton) -> Option<&Option<PointerButton>> {
        self.mapping.get(&mouse_button)
    }

    /// Set a mouse-to-pointer mapping.
    pub fn set_mapping(
        &mut self,
        mouse_button: MouseButton,
        pointer_button: Option<PointerButton>,
    ) {
        let mut clear = None;
        match pointer_button {
            Some(p) => {
                for (current_m, current_p) in self.mapping.iter() {
                    if current_p == &pointer_button {
                        if current_m == &mouse_button {
                            debug!(
                                "{:?} Mouse button already mapped to {:?} Pointer button ",
                                mouse_button, p
                            );
                            return;
                        } else {
                            debug!("{:?} Mouse button was mapped to {:?} Pointer button, and will be set to None to avoid conflicts.", mouse_button,p);
                            clear = Some(mouse_button);
                        }
                    }
                }
                debug!(
                    "Setting {:?} Mouse button to {:?} Pointer button ",
                    mouse_button, p
                )
            }
            None => debug!("Setting {:?} Mouse button to None", mouse_button),
        };
        if let Some(m) = clear {
            self.mapping.insert(m, None);
        }
        self.mapping.insert(mouse_button, pointer_button);
    }
}

/// Spawns the default mouse pointer.
pub fn spawn_mouse_pointer(mut commands: Commands) {
    commands.init_resource::<MouseButtonSettings>();
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
    mouse_button_settings: Res<MouseButtonSettings>,
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
        let button = match mouse_button_settings.get_mapping(input.button) {
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
