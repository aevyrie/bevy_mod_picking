use crate::{PickableMesh, PickingCamera, PickingPluginState, RayCastPluginState, Selection};
use bevy::{prelude::*, ui::FocusPolicy};

#[allow(clippy::type_complexity)]
pub fn mesh_focus(
    mut state: ResMut<PickingPluginState>,
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    pick_source_query: Query<&PickingCamera>,
    mut interaction_set: QuerySet<(
        Query<(&mut Interaction, Option<&FocusPolicy>, Entity), With<PickableMesh>>, //q0
        Query<&Interaction, With<Node>>,                                             //q1
    )>,
) {
    if !state.enabled {
        return;
    }

    let mut hovered_entity = None;

    // If anyting in the UI is being interacted with, set all pick interactions to none and exit
    for ui_interaction in interaction_set.q1().iter() {
        if *ui_interaction != Interaction::None {
            for (mut interaction, _, _) in &mut interaction_set.q0_mut().iter_mut() {
                if *interaction != Interaction::None {
                    *interaction = Interaction::None;
                }
            }
            state.paused_for_ui = true;
            return;
        } else {
            state.paused_for_ui = false;
        }
    }

    if mouse_button_input.just_released(MouseButton::Left) || touches_input.just_released(0) {
        for (mut interaction, _, _) in &mut interaction_set.q0_mut().iter_mut() {
            if *interaction == Interaction::Clicked {
                *interaction = Interaction::None;
            }
        }
    }

    let mouse_clicked =
        mouse_button_input.just_pressed(MouseButton::Left) || touches_input.just_released(0);

    for pick_source in pick_source_query.iter() {
        // There is at least one entity under the cursor
        if let Some(picks) = pick_source.intersect_list() {
            for (topmost_entity, _intersection) in picks.iter() {
                if let Ok((mut interaction, focus_policy, _entity)) =
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

        for (mut interaction, _, entity) in &mut interaction_set.q0_mut().iter_mut() {
            if Some(entity) != hovered_entity && *interaction == Interaction::Hovered {
                *interaction = Interaction::None;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn mesh_focus_debug_system(
    state: Res<RayCastPluginState>,
    query: Query<
        (&Interaction, &Selection, Entity),
        (
            Or<(Changed<Interaction>, Changed<Selection>)>,
            With<PickableMesh>,
        ),
    >,
) {
    if !state.enabled {
        return;
    }
    for (interaction, selection, entity) in query.iter() {
        println!(
            "ENTITY:{:?} INTERACTION:{:?} SELECTION:{:?}",
            entity,
            interaction,
            selection.selected()
        );
    }
}
