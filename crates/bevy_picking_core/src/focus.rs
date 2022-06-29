use crate::{
    events::Just,
    hit::CursorOver,
    input::{CursorClick, CursorId},
    CursorEvent, PickableTarget,
};
use bevy::{prelude::*, ui::FocusPolicy, utils::HashSet};

#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct CursorInteraction {
    pub(crate) hovered: HashSet<CursorId>,
    pub(crate) clicked: HashSet<CursorId>,
}
impl CursorInteraction {
    pub fn is_hovered(&self, cursor: &CursorId) -> bool {
        self.hovered.contains(cursor)
    }

    pub fn is_clicked(&self, cursor: &CursorId) -> bool {
        self.clicked.contains(cursor)
    }

    pub fn is_hovered_any(&self) -> bool {
        !self.hovered.is_empty()
    }

    pub fn is_clicked_any(&self) -> bool {
        !self.clicked.is_empty()
    }
}

#[allow(clippy::type_complexity)]
pub fn update_focus(
    mut cursors: Query<
        (
            &CursorId,
            &CursorClick,
            ChangeTrackers<CursorClick>,
            &mut CursorOver,
        ),
        Or<(Changed<CursorOver>, Changed<CursorClick>)>,
    >,
    focus: Query<Option<&FocusPolicy>, With<PickableTarget>>,
    mut interaction: Query<&mut CursorInteraction, With<PickableTarget>>,
    mut events: EventWriter<CursorEvent>,
) {
    for (&cursor_id, click, click_tracker, mut over) in cursors.iter_mut() {
        over.swap_unblocked();
        let mut all_entities = over.entities.clone();
        for entity in all_entities.drain(0..) {
            let mut interaction = interaction.get_mut(entity).unwrap();

            if !over.unblocked_prev.contains(&entity) {
                interaction.hovered.insert(cursor_id);
                events.send(CursorEvent::new(entity, cursor_id, Just::Entered));
            }

            if click.is_clicked && click_tracker.is_changed() {
                interaction.clicked.insert(cursor_id);
                events.send(CursorEvent::new(entity, cursor_id, Just::Down));
            } else if !click.is_clicked && click_tracker.is_changed() {
                interaction.clicked.remove(&cursor_id);
                events.send(CursorEvent::new(entity, cursor_id, Just::Up));
            }

            over.unblocked_current.push(entity);

            if let Ok(Some(_policy @ FocusPolicy::Pass)) = focus.get(entity) {
                continue;
            } else {
                break;
            }
        }
        for &entity in over.unblocked_prev.iter() {
            if !over.unblocked_current.contains(&entity) {
                interaction
                    .get_mut(entity)
                    .unwrap()
                    .hovered
                    .remove(&cursor_id);
                events.send(CursorEvent::new(entity, cursor_id, Just::Exited));
            }
        }
    }
}
