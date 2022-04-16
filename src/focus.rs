use crate::{PausedForBlockers, PickableMesh, PickingCamera};
use bevy::{prelude::*, ui::FocusPolicy};

/// Tracks the current hover state to be used with change tracking in the events system.
///
/// # Requirements
///
/// An entity with the `Hover` component must also have an [Interaction] component.
#[derive(Component, Debug, Default, Copy, Clone)]
pub struct Hover {
    hovered: bool,
}

impl Hover {
    pub fn hovered(&self) -> bool {
        self.hovered
    }
}

/// Marker component for entities that, whenever their [Interaction] component is anything other
/// than `None`, will suspend highlighting and selecting [PickableMesh]s. Bevy UI [Node]s have this
/// behavior by default.
#[derive(Component)]
pub struct PickingBlocker;

#[allow(clippy::type_complexity)]
pub fn pause_for_picking_blockers(
    mut paused: ResMut<PausedForBlockers>,
    mut interactions: ParamSet<(
        Query<
            (
                &mut Interaction,
                Option<&mut Hover>,
                Option<&FocusPolicy>,
                Entity,
            ),
            With<PickableMesh>,
        >,
        // UI nodes are picking blockers by default.
        Query<&Interaction, Or<(With<Node>, With<PickingBlocker>)>>,
    )>,
) {
    for ui_interaction in interactions.p1().iter() {
        if *ui_interaction != Interaction::None {
            for (mut interaction, hover, _, _) in &mut interactions.p0().iter_mut() {
                if *interaction != Interaction::None {
                    *interaction = Interaction::None;
                }
                if let Some(mut hover) = hover {
                    if hover.hovered {
                        hover.hovered = false;
                    }
                }
            }
            paused.0 = true;
            return;
        } else {
            paused.0 = false;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn mesh_focus(
    paused: Option<Res<PausedForBlockers>>,
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    pick_source_query: Query<&PickingCamera>,
    mut interactions: Query<
        (
            &mut Interaction,
            Option<&mut Hover>,
            Option<&FocusPolicy>,
            Entity,
        ),
        With<PickableMesh>,
    >,
) {
    if let Some(paused) = paused {
        if paused.0 {
            return;
        }
    }

    let mut hovered_entity = None;

    if mouse_button_input.just_released(MouseButton::Left)
        || touches_input.iter_just_released().next().is_some()
    {
        for (mut interaction, _, _, _) in &mut interactions.iter_mut() {
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
                    interactions.get_mut(*topmost_entity)
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

        for (mut interaction, hover, _, entity) in &mut interactions.iter_mut() {
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
