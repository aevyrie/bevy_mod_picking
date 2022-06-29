use bevy::prelude::*;
use bevy_picking_core::input::{CursorClick, CursorId, CursorInput};

/// Unsurprising default picking inputs
pub fn default_picking_inputs(
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    touches: Res<Touches>,
    mut cursor_query: Query<(&CursorId, &mut CursorInput, &mut CursorClick)>,
) {
    let multiselect = keyboard.any_pressed([
        KeyCode::LControl,
        KeyCode::RControl,
        KeyCode::LShift,
        KeyCode::RShift,
    ]);

    for (&id, mut input, mut click) in cursor_query.iter_mut() {
        if input.as_ref().multiselect != multiselect {
            input.multiselect = multiselect;
        }

        match id {
            CursorId::Touch(touch_id) => {
                if touches.get_pressed(touch_id).is_some() && !click.as_ref().clicked {
                    click.clicked = true;
                } else if click.as_ref().clicked {
                    click.clicked = false;
                }
            }
            CursorId::Mouse => {
                let pressed = mouse.pressed(MouseButton::Left);
                if pressed && !click.as_ref().clicked {
                    click.clicked = true;
                }
                if !pressed && click.as_ref().clicked {
                    click.clicked = false;
                }
            }
            _ => (),
        }
    }
}
