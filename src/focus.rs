use crate::{PickableMesh, PickingCamera, PickingPluginState};
use bevy::{prelude::*, ui::FocusPolicy};

/// Tracks the current hover state to be used with change tracking in the events system.
///
/// # Requirements
///
/// An entity with the `Hover` component must also have an [Interaction] component.
#[derive(Debug, Copy, Clone)]
pub struct Hover {
    hovered: bool,
}

impl Hover {
    pub fn hovered(&self) -> bool {
        self.hovered
    }
}

impl Default for Hover {
    fn default() -> Self {
        Hover { hovered: false }
    }
}

#[allow(clippy::type_complexity)]
pub fn mesh_focus(
    mut state: ResMut<PickingPluginState>,
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    pick_source_query: Query<&PickingCamera>,
    mut interaction_set: QuerySet<(
        Query<
            (
                &mut Interaction,
                Option<&mut Hover>,
                Option<&FocusPolicy>,
                Entity,
            ),
            With<PickableMesh>,
        >, //q0
        Query<&Interaction, With<Node>>, //q1
    )>,
) {
    if !state.enabled {
        return;
    }

    let mut hovered_entity = None;

    // If anything in the UI is being interacted with, set all pick interactions to none and exit
    for ui_interaction in interaction_set.q1().iter() {
        if *ui_interaction != Interaction::None {
            for (mut interaction, hover, _, _) in &mut interaction_set.q0_mut().iter_mut() {
                if *interaction != Interaction::None {
                    *interaction = Interaction::None;
                }
                if let Some(mut hover) = hover {
                    if hover.hovered {
                        hover.hovered = false;
                    }
                }
            }
            state.paused_for_ui = true;
            return;
        } else {
            state.paused_for_ui = false;
        }
    }

    if mouse_button_input.just_released(MouseButton::Left)
        || touches_input.iter_just_released().next().is_some()
    {
        for (mut interaction, _, _, _) in &mut interaction_set.q0_mut().iter_mut() {
            if *interaction == Interaction::Clicked {
                *interaction = Interaction::None;
            }
        }
    }

    let mouse_clicked = mouse_button_input.just_pressed(MouseButton::Left)
        || touches_input.iter_just_pressed().next().is_some();
    for pick_source in pick_source_query.iter() {
        // There is at least one entity under the cursor
        if let Some(picks) = pick_source.intersect_list() {
            for (topmost_entity, _intersection) in picks.iter() {
                if let Ok((mut interaction, _hover, focus_policy, _entity)) =
                    interaction_set.q0_mut().get_mut(*topmost_entity)
                {
                    if mouse_clicked {
                        if *interaction != Interaction::Clicked {
                            *interaction = Interaction::Clicked;
                        }
                    } else if *interaction == Interaction::None {
                        *interaction = Interaction::Hovered;
                    }

                    hovered_entity = Some(*topmost_entity);

                    match focus_policy.cloned().unwrap_or(FocusPolicy::Block) {
                        FocusPolicy::Block => {
                            break;
                        }
                        FocusPolicy::Pass => { /* allow the next node to be hovered/clicked */ }
                    }
                }
            }
        }

        for (mut interaction, hover, _, entity) in &mut interaction_set.q0_mut().iter_mut() {
            if Some(entity) != hovered_entity && *interaction == Interaction::Hovered {
                *interaction = Interaction::None;
            }
            if Some(entity) == hovered_entity {
                if let Some(mut hover) = hover {
                    if !hover.hovered {
                        hover.hovered = true;
                    }
                }
            } else if let Some(mut hover) = hover {
                if hover.hovered {
                    hover.hovered = false;
                }
            }
        }
    }
}
