use crate::{PickingCamera, UpdatePicks};
use bevy::{
    prelude::*,
    render::camera::{Camera, RenderTarget},
};
use bevy_mod_raycast::RayCastMethod;

/// Update Screenspace ray cast sources with the current mouse position
pub fn update_pick_source_positions(
    touches_input: Res<Touches>,
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
    mut cursor: EventReader<CursorMoved>,
    mut pick_source_query: Query<(
        &mut PickingCamera,
        Option<&mut UpdatePicks>,
        Option<&Camera>,
    )>,
) {
    for (mut pick_source, option_update_picks, option_camera) in &mut pick_source_query.iter_mut() {
        let (mut update_picks, cursor_latest) = match get_inputs(
            &windows,
            &images,
            option_camera,
            option_update_picks,
            &mut cursor,
            &touches_input,
        ) {
            Some(value) => value,
            None => return,
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

fn get_inputs<'a>(
    windows: &Res<Windows>,
    images: &Res<Assets<Image>>,
    option_camera: Option<&Camera>,
    option_update_picks: Option<Mut<'a, UpdatePicks>>,
    cursor: &mut EventReader<CursorMoved>,
    touches_input: &Res<Touches>,
) -> Option<(Mut<'a, UpdatePicks>, Option<Vec2>)> {
    let camera = option_camera?;
    let update_picks = option_update_picks?;
    let height = camera.target.get_logical_size(windows, images)?.y;
    let cursor_latest = match cursor.iter().last() {
        Some(cursor_moved) => {
            if let RenderTarget::Window(window) = camera.target {
                if cursor_moved.id == window {
                    Some(cursor_moved.position)
                } else {
                    None
                }
            } else {
                None
            }
        }
        None => touches_input.iter().last().map(|touch| {
            Vec2::new(
                touch.position().x as f32,
                height - touch.position().y as f32,
            )
        }),
    };
    Some((update_picks, cursor_latest))
}
