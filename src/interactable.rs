use crate::PickingRaycastSet;
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_mod_raycast::RayCastSource;
use std::iter::FromIterator;

#[derive(Debug, Copy, Clone)]
pub enum HoverEvents {
    None,
    JustEntered,
    JustExited,
}

impl Default for HoverEvents {
    fn default() -> Self {
        HoverEvents::None
    }
}

impl HoverEvents {
    pub fn is_none(&self) -> bool {
        matches!(self, HoverEvents::None)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseButtonEvents {
    None,
    JustPressed,
    JustReleased,
}

impl MouseButtonEvents {
    pub fn is_none(&self) -> bool {
        matches!(self, MouseButtonEvents::None)
    }
}

#[derive(Debug)]
pub struct InteractableMesh {
    hover_events: HoverEvents,
    mouse_down_events: HashMap<MouseButton, MouseButtonEvents>,
    hovering: bool,
    mouse_down: Vec<MouseButton>,
    watched_mouse_inputs: HashSet<MouseButton>,
}

impl Default for InteractableMesh {
    fn default() -> Self {
        InteractableMesh {
            hover_events: HoverEvents::None,
            mouse_down_events: HashMap::default(),
            hovering: false,
            mouse_down: Vec::default(),
            watched_mouse_inputs: HashSet::from_iter(
                [MouseButton::Left, MouseButton::Right, MouseButton::Middle]
                    .iter()
                    .cloned(),
            ),
        }
    }
}

impl InteractableMesh {
    pub fn with_mouse_button(self, button: MouseButton) -> Self {
        let mut new_buttons = self.watched_mouse_inputs;
        new_buttons.insert(button);
        InteractableMesh {
            watched_mouse_inputs: new_buttons,
            ..self
        }
    }
    /// Returns the current hover event state of the InteractableMesh in the provided group.
    pub fn hover_event(&self) -> HoverEvents {
        self.hover_events
    }
    /// Returns true iff the InteractableMesh is the topost entity in the specified group.
    pub fn hovering(&self) -> bool {
        self.hovering
    }
    /// Returns the current mousedown event state of the InteractableMesh in the provided group.
    pub fn mouse_down_event_list(&self) -> &HashMap<MouseButton, MouseButtonEvents> {
        &self.mouse_down_events
    }
    pub fn mouse_down_event(&self, button: MouseButton) -> Result<MouseButtonEvents, String> {
        self.mouse_down_events.get(&button).copied().ok_or(format!(
            "MouseButton {:?} not found in this InteractableMesh",
            button
        ))
    }
    pub fn just_pressed(&self, button: MouseButton) -> bool {
        matches!(
            self.mouse_down_events.get(&button),
            Some(&MouseButtonEvents::JustPressed)
        )
    }
}

pub fn generate_hover_events(
    pick_source_query: Query<&RayCastSource<PickingRaycastSet>>,
    mut interactable_query: Query<(&mut InteractableMesh, Entity)>,
) {
    for pick_source in pick_source_query.iter() {
        match pick_source.intersect_top() {
            // There is at last one entity under the cursor
            Some(top_pick) => {
                let top_entity = top_pick.0;
                for (mut interactable, entity) in &mut interactable_query.iter_mut() {
                    let now_hovering = entity == top_entity;
                    let previously_hovered = interactable.hovering;
                    interactable.hover_events = {
                        if now_hovering && !previously_hovered {
                            HoverEvents::JustEntered
                        } else if !now_hovering && previously_hovered {
                            HoverEvents::JustExited
                        } else {
                            HoverEvents::None
                        }
                    };
                    interactable.hovering = now_hovering;
                }
            }
            // There are no entities under the cursor
            None => {
                for (mut interactable, _entity) in &mut interactable_query.iter_mut() {
                    let was_hovered = interactable.hovering;
                    interactable.hover_events = {
                        if was_hovered {
                            HoverEvents::JustExited
                        } else {
                            HoverEvents::None
                        }
                    };
                    interactable.hovering = false;
                }
            }
        }
    }
}

pub fn generate_click_events(
    // Resources
    mouse_inputs: Res<Input<MouseButton>>,
    // Queries
    mut interactable_query: Query<&mut InteractableMesh>,
) {
    interactable_query.iter_mut().for_each(|mut interactable| {
        let events: Vec<(MouseButton, MouseButtonEvents)> = interactable
            .watched_mouse_inputs
            .iter()
            .map(|button| {
                (
                    *button,
                    if !interactable.hovering() {
                        MouseButtonEvents::None
                    } else if mouse_inputs.just_released(*button) {
                        MouseButtonEvents::JustReleased
                    } else if mouse_inputs.just_pressed(*button) {
                        MouseButtonEvents::JustPressed
                    } else {
                        MouseButtonEvents::None
                    },
                )
            })
            .collect();

        events.iter().for_each(|(button, event)| {
            interactable.mouse_down_events.insert(*button, *event);
        });
    });
}
