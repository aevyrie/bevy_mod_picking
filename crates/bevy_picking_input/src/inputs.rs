use bevy::prelude::*;
use bevy_picking_core::input::{CursorId, CursorInput};

/// Unsurprising default picking inputs
pub fn default_picking_inputs(
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut cursor_query: Query<(&CursorId, &mut CursorInput)>,
    touches: Res<Touches>,
) {
    let multiselect = keyboard.any_pressed([
        KeyCode::LControl,
        KeyCode::RControl,
        KeyCode::LShift,
        KeyCode::RShift,
    ]);

    for (&id, mut cursor) in cursor_query.iter_mut() {
        cursor.multiselect = multiselect;

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
