use crate::{
    backend::CursorOver,
    events::Just,
    input::{CursorClick, CursorId},
    interaction::CursorInteraction,
    CursorEvent,
};
use bevy::{prelude::*, ui::FocusPolicy};

#[allow(clippy::type_complexity)]
pub fn update_focus(
    cursors: Query<
        (&CursorId, &CursorClick, &CursorOver),
        Or<(Changed<CursorClick>, Changed<CursorOver>)>,
    >,
    focus: Query<&FocusPolicy>,
    mut interaction: Query<(Entity, &mut CursorInteraction)>,
    mut events: EventWriter<CursorEvent>,
) {
    for (&cursor_id, click, over) in cursors.iter() {
        let mut hovered = Vec::new();
        for entity in over.entities.iter() {
            hovered.push(*entity);
            if let Ok(_policy @ FocusPolicy::Pass) = focus.get(*entity) {
                continue;
            } else {
                break;
            }
        }

        for (entity, mut interaction) in interaction.iter_mut() {
            if hovered.contains(&entity) && !interaction.is_hovered(&cursor_id) {
                interaction.hovered.insert(cursor_id);
                events.send(CursorEvent::new(entity, cursor_id, Just::Entered));
            } else if !hovered.contains(&entity) && interaction.is_hovered(&cursor_id) {
                interaction.hovered.remove(&cursor_id);
                events.send(CursorEvent::new(entity, cursor_id, Just::Exited));
            }

            if click.is_clicked && !interaction.is_clicked(&cursor_id) {
                interaction.clicked.insert(cursor_id);
                events.send(CursorEvent::new(entity, cursor_id, Just::Down));
            } else if !click.is_clicked && interaction.is_clicked(&cursor_id) {
                interaction.clicked.remove(&cursor_id);
                events.send(CursorEvent::new(entity, cursor_id, Just::Up));
            }
        }
    }
}
