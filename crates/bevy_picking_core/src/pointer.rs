//! Types and systems for pointer inputs, such as position and buttons.

use bevy::{
    prelude::*,
    render::camera::RenderTarget,
    utils::{HashMap, Uuid},
};
use std::fmt::Debug;

/// Identifies a unique pointer entity. `Mouse` and `Touch` pointers are automatically spawned.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Component)]
pub enum PointerId {
    /// A touch input, normally numbered by incoming window touch events from `winit`.
    Touch(u64),
    /// The mouse pointer.
    Mouse,
    /// A custom, uniquely identified pointer. Useful for mocking inputs or implementing a software
    /// controlled cursor.
    Custom(Uuid),
}
impl PointerId {
    /// Returns true if the pointer is a touch input.
    pub fn is_touch(&self) -> bool {
        matches!(self, PointerId::Touch(_))
    }
    /// Returns true if the pointer is the mouse.
    pub fn is_mouse(&self) -> bool {
        matches!(self, PointerId::Mouse)
    }
    /// Returns true if the pointer is a custom input.
    pub fn is_custom(&self) -> bool {
        matches!(self, PointerId::Custom(_))
    }
    /// Returns the touch id if the pointer is a touch input.
    pub fn get_touch_id(&self) -> Option<u64> {
        if let PointerId::Touch(id) = self {
            Some(*id)
        } else {
            None
        }
    }
}

/// Maps pointers to their entity for easy lookups.
#[derive(Debug, Clone, Default, Resource)]
pub struct PointerMap {
    inner: HashMap<PointerId, Entity>,
}

impl PointerMap {
    /// Get the [`Entity`] of the supplied [`PointerId`].
    pub fn get_entity(&self, pointer_id: PointerId) -> Option<Entity> {
        self.inner.get(&pointer_id).copied()
    }
}

/// Update the [`PointerMap`] resource with the current frame's data.
pub fn update_pointer_map(pointers: Query<(Entity, &PointerId)>, mut map: ResMut<PointerMap>) {
    map.inner.clear();
    for (entity, id) in &pointers {
        map.inner.insert(*id, entity);
    }
}

/// Tracks the state of the pointer's buttons in response to [`InputPress`]s.
#[derive(Debug, Default, Clone, Component, Reflect, PartialEq, Eq)]
pub struct PointerPress {
    primary: bool,
    secondary: bool,
    middle: bool,
}
impl PointerPress {
    /// Returns true if the primary pointer button is pressed.
    #[inline]
    pub fn is_primary_pressed(&self) -> bool {
        self.primary
    }

    /// Returns true if the secondary pointer button is pressed.
    #[inline]
    pub fn is_secondary_pressed(&self) -> bool {
        self.secondary
    }

    /// Returns true if the middle (tertiary) pointer button is pressed.
    #[inline]
    pub fn is_middle_pressed(&self) -> bool {
        self.middle
    }

    /// Returns true if any pointer button is pressed.
    #[inline]
    pub fn is_any_pressed(&self) -> bool {
        self.primary || self.middle || self.secondary
    }
}

/// Pointer input event for button presses. Fires when a pointer button changes state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputPress {
    /// ID of the pointer for this event.
    pointer_id: PointerId,
    /// Direction of the button press.
    press: PressDirection,
    /// Identifies the pointer button changing in this event.
    button: PointerButton,
}
impl InputPress {
    /// Create a new pointer button down event.
    pub fn new_down(id: PointerId, button: PointerButton) -> InputPress {
        Self {
            pointer_id: id,
            press: PressDirection::Down,
            button,
        }
    }

    /// Create a new pointer button up event.
    pub fn new_up(id: PointerId, button: PointerButton) -> InputPress {
        Self {
            pointer_id: id,
            press: PressDirection::Up,
            button,
        }
    }

    /// Returns true if the `button` of this pointer was just pressed.
    #[inline]
    pub fn is_just_down(&self, button: PointerButton) -> bool {
        self.button == button && self.press == PressDirection::Down
    }

    /// Returns true if the `button` of this pointer was just released.
    #[inline]
    pub fn is_just_up(&self, button: PointerButton) -> bool {
        self.button == button && self.press == PressDirection::Up
    }

    /// Receives [`InputPress`] events and updates corresponding [`PointerPress`] components.
    pub fn receive(
        mut events: EventReader<InputPress>,
        mut pointers: Query<(&PointerId, &mut PointerPress)>,
    ) {
        for input_press_event in events.iter() {
            pointers.for_each_mut(|(pointer_id, mut pointer)| {
                if *pointer_id == input_press_event.pointer_id {
                    let is_down = input_press_event.press == PressDirection::Down;
                    match input_press_event.button {
                        PointerButton::Primary => pointer.primary = is_down,
                        PointerButton::Secondary => pointer.secondary = is_down,
                        PointerButton::Middle => pointer.middle = is_down,
                    }
                }
            })
        }
    }

    /// Gets the [`PointerId`] of the event.
    pub fn pointer_id(&self) -> PointerId {
        self.pointer_id
    }

    /// Gets the [`PressDirection`] of the event.
    pub fn direction(&self) -> PressDirection {
        self.press
    }

    /// Gets the [`PointerButton`] of the event.
    pub fn button(&self) -> PointerButton {
        self.button
    }
}

/// The stage of the pointer button press event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressDirection {
    /// The pointer button was just pressed
    Down,
    /// The pointer button was just released
    Up,
}

/// The button that was just pressed or released
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum PointerButton {
    /// The primary pointer button
    Primary,
    /// The secondary pointer button
    Secondary,
    /// The tertiary pointer button
    Middle,
}

impl PointerButton {
    /// Iterator over all buttons that a pointer can have.
    pub fn all_buttons() -> impl Iterator<Item = PointerButton> {
        [Self::Primary, Self::Secondary, Self::Middle].into_iter()
    }
}

/// Component that tracks a pointer's current [`Location`].
#[derive(Debug, Default, Clone, Component, Reflect, PartialEq)]
pub struct PointerLocation {
    /// The [`Location`] of the pointer. Note that a location is both the target, and the position
    /// on the target.
    pub location: Option<Location>,
}
impl PointerLocation {
    /// Returns `Some(&`[`Location`]`)` if the pointer is active, or `None` if the pointer is
    /// inactive.
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
}

/// Pointer input event for pointer moves. Fires when a pointer changes location.
#[derive(Debug, Clone)]
pub struct InputMove {
    pointer_id: PointerId,
    location: Location,
}
impl InputMove {
    /// Create a new [`InputMove`] event.
    pub fn new(id: PointerId, location: Location) -> InputMove {
        Self {
            pointer_id: id,
            location,
        }
    }

    /// Receives [`InputMove`] events and updates corresponding [`PointerLocation`] components.
    pub fn receive(
        mut events: EventReader<InputMove>,
        mut pointers: Query<(&PointerId, &mut PointerLocation)>,
    ) {
        for event_pointer in events.iter() {
            pointers.for_each_mut(|(id, mut pointer)| {
                if *id == event_pointer.pointer_id {
                    pointer.location = Some(event_pointer.location.to_owned());
                }
            })
        }
    }

    /// Returns the [`PointerId`] of this event.
    pub fn pointer_id(&self) -> PointerId {
        self.pointer_id
    }

    /// Returns the [`Location`] of this event.
    pub fn location(&self) -> &Location {
        &self.location
    }
}

/// The location of a pointer, including the current [`RenderTarget`], and the x/y position of the
/// pointer on this render target.
///
/// Note that a pointer can move freely between render targets.
#[derive(Debug, Clone, Component, Reflect, FromReflect, PartialEq)]
pub struct Location {
    /// The [`RenderTarget`] associated with the pointer, usually a window.
    #[reflect(ignore)]
    pub target: RenderTarget,
    /// The position of the pointer in the `target`.
    pub position: Vec2,
}
impl Location {
    /// Returns `true` if this pointer's [`Location`] is within the [`Camera`]'s viewport.
    ///
    /// Note this returns `false` if the location and camera have different [`RenderTarget`]s.
    #[inline]
    pub fn is_in_viewport(&self, camera: &Camera, windows: &Windows) -> bool {
        if !self.is_same_target(camera) {
            return false;
        }

        let window = if let RenderTarget::Window(id) = self.target {
            if let Some(w) = windows.get(id) {
                w
            } else {
                return false;
            }
        } else {
            return false;
        };

        let position = Vec2::new(self.position.x, window.height() - self.position.y);

        camera
            .logical_viewport_rect()
            .map(|(min, max)| {
                (position - min).min_element() >= 0.0 && (position - max).max_element() <= 0.0
            })
            .unwrap_or(false)
    }

    /// Returns `true` if this [`Location`] and the [`Camera`] have the same [`RenderTarget`].
    #[inline]
    pub fn is_same_target(&self, camera: &Camera) -> bool {
        self.target == camera.target
    }
}
