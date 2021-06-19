use crate::{PickingCamera, UpdatePicks};
use bevy::{prelude::*, render::camera::Camera};
use bevy_mod_raycast::RayCastMethod;

/// Update Screenspace ray cast sources with the current mouse position
pub fn update_pick_source_positions(
    touches_input: Res<Touches>,
    windows: Res<Windows>,
    mut cursor: EventReader<CursorMoved>,
    mut pick_source_query: Query<(
        &mut PickingCamera,
        Option<&mut UpdatePicks>,
        Option<&Camera>,
    )>,
) {
    for (mut pick_source, option_update_picks, option_camera) in &mut pick_source_query.iter_mut() {
        let camera = match option_camera {
            Some(camera) => camera,
            None => panic!("The PickingCamera entity has no associated Camera component"),
        };
        let mut update_picks = match option_update_picks {
            Some(update_picks) => update_picks,
            None => panic!("The PickingCamera entity has no associated UpdatePicks component"),
        };
        let cursor_latest = match cursor.iter().last() {
            Some(cursor_moved) => {
                if cursor_moved.id == camera.window {
                    Some(cursor_moved.position)
                } else {
                    None
                }
            }
            None => touches_input.iter().last().map(|touch| {
                Vec2::new(
                    touch.position().x,
                    windows
                        .get(camera.window)
                        .expect("PickingCamera window does not exist")
                        .height(),
                )
            }),
        };
        match *update_picks {
            UpdatePicks::EveryFrame(cached_cursor_pos) => {
                match cursor_latest {
                    Some(cursor_moved) => {
                        pick_source.cast_method = RayCastMethod::Screenspace(cursor_moved);
                        *update_picks = UpdatePicks::EveryFrame(cursor_moved);
                    }
                    None => pick_source.cast_method = RayCastMethod::Screenspace(cached_cursor_pos),
                };
            }
            UpdatePicks::OnMouseEvent => match cursor_latest {
                Some(cursor_moved) => {
                    pick_source.cast_method = RayCastMethod::Screenspace(cursor_moved)
                }
                None => continue,
            },
        };
    }
}
