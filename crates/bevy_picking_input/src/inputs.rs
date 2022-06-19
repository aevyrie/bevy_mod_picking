use bevy::prelude::*;
use bevy_picking_core::picking::cursor::{Cursor, CursorId, MultiSelect};

/// Unsurprising default picking inputs
pub fn default_picking_inputs(
    mut multiselect: ResMut<MultiSelect>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut cursor_query: Query<(&CursorId, &mut Cursor)>,
    touches: Res<Touches>,
) {
    multiselect.active = keyboard.any_pressed([
        KeyCode::LControl,
        KeyCode::RControl,
        KeyCode::LShift,
        KeyCode::RShift,
    ]);

    for (&id, mut cursor) in cursor_query.iter_mut() {
        match id {
            CursorId::Touch(id) => {
                if touches.get_pressed(id).is_some() {
                    cursor.clicked = true;
                }
            }
            CursorId::Mouse => {
                if mouse.pressed(MouseButton::Left) {
                    cursor.clicked = true;
                }
            }
            _ => (),
        }
    }
}
