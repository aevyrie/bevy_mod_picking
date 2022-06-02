use crate::{PausedForBlockers, PickableTarget, PickingInput};
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
/// than `None`, will suspend highlighting and selecting [PickableTarget]s. Bevy UI [Node]s have this
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
            With<PickableTarget>,
        >,
        // UI nodes are picking blockers by default.
        Query<&Interaction, Or<(With<Node>, With<PickingBlocker>)>>,
    )>,
) {
    paused.0 = false;
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
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_focus(
    inputs: Res<PickingInput>,
    paused: Option<Res<PausedForBlockers>>,
    mut interactions: Query<
        (
            &mut Interaction,
            Option<&mut Hover>,
            Option<&FocusPolicy>,
            Entity,
        ),
        With<PickableTarget>,
    >,
) {
    if let Some(paused) = paused {
        if paused.0 {
            return;
        }
    }

    //
    // if mouse_button_input.just_released(MouseButton::Left)
    // || touches_input.iter_just_released().next().is_some()

    // Reset all click interactions
    if inputs.pick_event {
        for (mut interaction, _, _, _) in &mut interactions.iter_mut() {
            if *interaction == Interaction::Clicked {
                *interaction = Interaction::None;
            }
        }
    }

    // let mouse_clicked = mouse_button_input.just_pressed(MouseButton::Left)
    //     || touches_input.iter_just_pressed().next().is_some();

    for entity in &inputs.hovered_entities {
        if let Ok((mut interaction, _, focus_policy, _)) = interactions.get_mut(*entity) {
            if inputs.pick_event {
                if *interaction != Interaction::Clicked {
                    *interaction = Interaction::Clicked;
                }
            } else if *interaction == Interaction::None {
                *interaction = Interaction::Hovered;
            }

            match focus_policy.cloned().unwrap_or(FocusPolicy::Block) {
                FocusPolicy::Block => {
                    break; // Prevents selecting anything further away
                }
                // Allows the next furthest entity to be clicked
                FocusPolicy::Pass => (),
            }
        }
    }

    for (mut interaction, hover, _, entity) in &mut interactions.iter_mut() {
        if inputs.hovered_entities.contains(&entity) {
            if let Some(mut hover) = hover {
                if !hover.hovered {
                    hover.hovered = true;
                }
            }
        // The following if statements are true only when the entity is not hovered
        } else if *interaction == Interaction::Hovered {
            *interaction = Interaction::None;
        } else if let Some(mut hover) = hover {
            if hover.hovered {
                hover.hovered = false;
            }
        }
    }
}
