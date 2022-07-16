use bevy::prelude::*;
use bevy_picking_core::input::{CursorClick, CursorId, CursorMultiSelect};

/// Unsurprising default picking inputs
pub fn default_picking_inputs(
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    touches: Res<Touches>,
    mut cursor_query: Query<(&CursorId, &mut CursorMultiSelect, &mut CursorClick)>,
) {
    let is_multiselect_down = keyboard.any_pressed([
        KeyCode::LControl,
        KeyCode::RControl,
        KeyCode::LShift,
        KeyCode::RShift,
    ]);

    for (&id, mut multiselect, mut click) in cursor_query.iter_mut() {
        if multiselect.as_ref().is_clicked != is_multiselect_down {
            multiselect.is_clicked = is_multiselect_down;
        }

        match id {
            CursorId::Touch(touch_id) => {
                let pressed = touches.get_pressed(touch_id).is_some();
                if pressed && !click.as_ref().is_clicked {
                    click.is_clicked = true;
                } else if !pressed && click.as_ref().is_clicked {
                    click.is_clicked = false;
                }
            }
            CursorId::Mouse => {
                let pressed = mouse.pressed(MouseButton::Left);
                if pressed && !click.as_ref().is_clicked {
                    click.is_clicked = true;
                } else if !pressed && click.as_ref().is_clicked {
                    click.is_clicked = false;
                }
            }
            _ => (),
        }
    }
}
