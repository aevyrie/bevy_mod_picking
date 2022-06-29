use crate::{hit::CursorHit, input::CursorClick, HoverEvent, PickableTarget, PickingEvent};
use bevy::{prelude::*, ui::FocusPolicy};

#[allow(clippy::type_complexity)]
pub fn update_focus(
    cursors: Query<(&CursorClick, ChangeTrackers<CursorClick>, &CursorHit)>,
    mut interactions: Query<(&mut Interaction, Option<&FocusPolicy>, Entity), With<PickableTarget>>,
    mut events: EventWriter<PickingEvent>,
) {
    let mut updated = Vec::new();

    for (click, click_track, hit) in cursors.iter() {
        // TODO: handle conflicting cursor interactions. e.g. if two cursors attempt to modify the
        // interaction state of a target entity, which one takes precedence?
        for entity in hit.entities.iter() {
            if let Ok((mut interaction, focus, _)) = interactions.get_mut(*entity) {
                updated.push(*entity);

                if *interaction.as_ref() == Interaction::None {
                    events.send(PickingEvent::Hover(HoverEvent::JustEntered(*entity)));
                }

                if click.clicked
                    && click_track.is_changed()
                    && *interaction.as_ref() != Interaction::Clicked
                {
                    *interaction = Interaction::Clicked;
                } else if *interaction.as_ref() == Interaction::None {
                    *interaction = Interaction::Hovered;
                }
                if let Some(_policy @ FocusPolicy::Block) = focus {
                    break; // Prevents interacting with anything further away
                }
            }
        }
    }

    for (mut interaction, _, entity) in &mut interactions.iter_mut() {
        if !updated.contains(&entity) {
            if *interaction.as_ref() != Interaction::None {
                *interaction = Interaction::None;
                events.send(PickingEvent::Hover(HoverEvent::JustLeft(entity)));
            }
        }
    }
}
