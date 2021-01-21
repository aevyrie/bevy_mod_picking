use super::{highlight::*, select::*, *};
use bevy::{prelude::*, utils::HashMap};

pub struct InteractablePickingPlugin;
impl Plugin for InteractablePickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(generate_hover_events.system())
            .add_system(generate_click_events.system())
            .add_system(select_mesh.system())
            .add_system(pick_highlighting.system());
    }
}

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
pub enum MouseDownEvents {
    None,
    MouseJustPressed,
    MouseJustReleased,
}

impl MouseDownEvents {
    pub fn is_none(&self) -> bool {
        matches!(self, MouseDownEvents::None)
    }
}

#[derive(Debug, Default)]
pub struct InteractableMesh {
    hover_events: HoverEvents,
    mouse_down_events: HashMap<MouseButton, MouseDownEvents>,
    hovering: bool,
    mouse_down: Vec<MouseButton>,
    watched_mouse_inputs: Vec<MouseButton>,
}

impl InteractableMesh {
    /// Returns the current hover event state of the InteractableMesh in the provided group.
    pub fn hover_event(&self) -> HoverEvents {
        self.hover_events
    }
    /// Returns true iff the InteractableMesh is the topost entity in the specified group.
    pub fn hover(&self) -> bool {
        self.hovering
    }
    /// Returns the current mousedown event state of the InteractableMesh in the provided group.
    pub fn mouse_down_event_list(&self) -> &HashMap<MouseButton, MouseDownEvents> {
        &self.mouse_down_events
    }
    pub fn mouse_down_event(&self, button: MouseButton) -> Result<MouseDownEvents, String> {
        self.mouse_down_events.get(&button).copied().ok_or(format!(
            "MouseButton {:?} not found in this InteractableMesh",
            button
        ))
    }
    pub fn just_pressed(&self, button: MouseButton) -> bool {
        matches!(
            self.mouse_down_events.get(&button),
            Some(&MouseDownEvents::MouseJustPressed)
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
                    let is_hovered = entity == top_entity;
                    let was_hovered = interactable.hovering;
                    interactable.hover_events = {
                        if is_hovered && !was_hovered {
                            HoverEvents::JustEntered
                        } else if !is_hovered && was_hovered {
                            HoverEvents::JustExited
                        } else {
                            HoverEvents::None
                        }
                    };
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
        let events: Vec<(MouseButton, MouseDownEvents)> = interactable
            .watched_mouse_inputs
            .iter()
            .map(|button| {
                (
                    *button,
                    if !interactable.hover() {
                        MouseDownEvents::None
                    } else if mouse_inputs.just_released(*button) {
                        MouseDownEvents::MouseJustReleased
                    } else if mouse_inputs.just_pressed(*button) {
                        MouseDownEvents::MouseJustPressed
                    } else {
                        MouseDownEvents::None
                    },
                )
            })
            .collect();

        events.iter().for_each(|(button, event)| {
            interactable.mouse_down_events.insert(*button, *event);
        });
    });
}
