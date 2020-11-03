use super::{highlight::*, select::*, *};
use bevy::prelude::*;
use std::collections::HashSet;

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

#[derive(Debug, Copy, Clone)]
pub enum MouseDownEvents {
    None,
    MouseJustPressed,
    MouseJustReleased,
}

#[derive(Debug)]
pub struct InteractableMesh {
    hover_events: HashMap<Group, HoverEvents>,
    mouse_down_events: HashMap<Group, HashMap<MouseButton, MouseDownEvents>>,
    hovering: HashMap<Group, bool>,
    mouse_down: HashMap<Group, Vec<MouseButton>>,
    watched_mouse_inputs: Vec<MouseButton>,
}

impl Default for InteractableMesh {
    fn default() -> Self {
        let mut hover_events = HashMap::new();
        let mut mouse_down_events = HashMap::new();
        let mut hovering = HashMap::new();
        let mut mouse_down = HashMap::new();
        let mut mouse_down_eventmap: HashMap<MouseButton, MouseDownEvents> = HashMap::new();
        let watched_mouse_inputs =
            Vec::from([MouseButton::Left, MouseButton::Right, MouseButton::Middle]);
        for button in &watched_mouse_inputs {
            mouse_down_eventmap.insert(*button, MouseDownEvents::None);
        }
        hover_events.insert(Group::default(), HoverEvents::None);
        mouse_down_events.insert(Group::default(), mouse_down_eventmap);
        hovering.insert(Group::default(), false);
        mouse_down.insert(Group::default(), Vec::new());
        InteractableMesh {
            hover_events,
            mouse_down_events,
            hovering,
            mouse_down,
            watched_mouse_inputs,
        }
    }
}

impl InteractableMesh {
    pub fn new(groups: Vec<Group>) -> Self {
        let mut hover_events = HashMap::new();
        let mut mouse_down_events = HashMap::new();
        let mut hovering = HashMap::new();
        let mut mouse_down = HashMap::new();
        let mut mouse_down_eventmap: HashMap<MouseButton, MouseDownEvents> = HashMap::new();
        let watched_mouse_inputs =
            Vec::from([MouseButton::Left, MouseButton::Right, MouseButton::Middle]);
        for button in &watched_mouse_inputs {
            mouse_down_eventmap.insert(*button, MouseDownEvents::None);
        }
        for group in &groups {
            hover_events.insert(*group, HoverEvents::None);
            mouse_down_events.insert(*group, mouse_down_eventmap.clone());
            hovering.insert(*group, false);
            mouse_down.insert(*group, Vec::new());
        }
        InteractableMesh {
            hover_events,
            mouse_down_events,
            hovering,
            mouse_down,
            watched_mouse_inputs,
        }
    }

    /// Returns the current hover event state of the InteractableMesh in the provided group.
    pub fn hover_event(&self, group: &Group) -> Result<&HoverEvents, String> {
        self.hover_events.get(group).ok_or(format!(
            "InteractableMesh does not belong to group {}",
            **group
        ))
    }

    /// Returns true iff the InteractableMesh is the topost entity in the specified group.
    pub fn hovered(&self, group: &Group) -> Result<&bool, String> {
        self.hovering.get(group).ok_or(format!(
            "InteractableMesh does not belong to group {}",
            **group
        ))
    }

    /// Returns the current mousedown event state of the InteractableMesh in the provided group.
    pub fn mouse_down_event_list(
        &self,
        group: &Group,
    ) -> Result<&HashMap<MouseButton, MouseDownEvents>, String> {
        self.mouse_down_events.get(group).ok_or(format!(
            "InteractableMesh does not belong to group {}",
            **group
        ))
    }

    pub fn mouse_down_event(
        &self,
        group: &Group,
        button: MouseButton,
    ) -> Result<&MouseDownEvents, String> {
        match self.mouse_down_events.get(group).ok_or(format!(
            "InteractableMesh does not belong to group {}",
            **group
        )) {
            Ok(event_map) => event_map.get(&button).ok_or(format!(
                "MouseButton {:?} not found in this InteractableMesh",
                button
            )),
            Err(e) => Err(e),
        }
    }

    /// Returns a HashSet of Groups in which the current InteractableMesh is just pressed
    pub fn groups_just_pressed(&self, button: MouseButton) -> HashSet<Group> {
        self.mouse_down_events
            .iter()
            .filter(|(_group, event_map)| {
                matches!(
                    event_map.get(&button),
                    Some(MouseDownEvents::MouseJustPressed)
                )
            })
            .map(|(group, _event_map)| *group)
            .collect()
    }
}

pub fn generate_hover_events(
    // Resources
    mut pick_state: ResMut<PickState>,
    // Queries
    mut interactable_query: Query<(&mut InteractableMesh, Entity)>,
) {
    for (group, intersection_list) in pick_state.ordered_pick_list_map.iter_mut() {
        match intersection_list.first() {
            // There is at last one entity under the cursor
            Some(top_pick) => {
                let top_entity = top_pick.0;
                for (mut interactable, entity) in &mut interactable_query.iter_mut() {
                    let is_hovered = entity == top_entity;
                    let was_hovered = interactable
                        .hovering
                        .insert(*group, is_hovered)
                        .unwrap_or(false);
                    let new_event = if is_hovered && !was_hovered {
                        HoverEvents::JustEntered
                    } else if !is_hovered && was_hovered {
                        HoverEvents::JustExited
                    } else {
                        HoverEvents::None
                    };
                    interactable.hover_events.insert(*group, new_event);
                }
            }
            // There are no entities under the cursor
            None => {
                for (mut interactable, _entity) in &mut interactable_query.iter_mut() {
                    let was_hovered = interactable
                        .hovering
                        .insert(*group, false)
                        .expect("Group missing");
                    let new_event = if was_hovered {
                        HoverEvents::JustExited
                    } else {
                        HoverEvents::None
                    };
                    interactable.hover_events.insert(*group, new_event);
                }
            }
        }
    }
}

pub fn generate_click_events(
    // Resources
    mouse_inputs: Res<Input<MouseButton>>,
    // Queries
    mut interactable_query: Query<(&mut InteractableMesh, &PickableMesh)>,
) {
    for (mut interactable, pickable) in interactable_query.iter_mut() {
        for group in &pickable.groups {
            match interactable.hovered(&group) {
                Ok(false) => continue,
                Err(_) => continue,
                _ => (),
            }
            let new_event = interactable
                .watched_mouse_inputs
                .iter()
                .map(|button| {
                    (
                        *button,
                        if mouse_inputs.just_released(*button) {
                            MouseDownEvents::MouseJustReleased
                        } else if mouse_inputs.just_pressed(*button) {
                            MouseDownEvents::MouseJustPressed
                        } else {
                            MouseDownEvents::None
                        },
                    )
                })
                .collect();
            interactable.mouse_down_events.insert(*group, new_event);
        }
    }
}
