use crate::{hit::CursorHit, input::CursorInput, PickableTarget};
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

#[allow(clippy::type_complexity)]
pub fn update_focus(
    cursors: Query<(&CursorInput, &CursorHit)>,
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
    let mut updated = Vec::new();

    for (input, hit) in cursors.iter() {
        // TODO: handle conflicting cursor interactions. e.g. if two cursors attempt to modify the
        // interaction state, which one takes precedence?

        for entity in hit.entities.iter() {
            if let Ok((mut interaction, hover, focus_policy, _)) = interactions.get_mut(*entity) {
                updated.push(*entity);
                if input.clicked {
                    *interaction = Interaction::Clicked;
                } else if *interaction == Interaction::None {
                    *interaction = Interaction::Hovered;
                }
                hover
                    .filter(|h| !h.as_ref().hovered)
                    .map(|mut hover| hover.hovered = true);
                if let Some(_policy @ FocusPolicy::Block) = focus_policy {
                    break; // Prevents interacting with anything further away
                }
            }
        }
    }

    for (mut interaction, hover, _, entity) in &mut interactions.iter_mut() {
        if !updated.contains(&entity) {
            if *interaction != Interaction::None {
                *interaction = Interaction::None;
            }
            hover
                .filter(|h| h.as_ref().hovered)
                .map(|mut hover| hover.hovered = false);
        }
    }
}
