use crate::{PickingCamera, UpdatePicks};
use bevy::{
    prelude::*,
    render::camera::{Camera, RenderTarget},
};
use bevy_mod_raycast::RaycastMethod;

/// Update Screenspace ray cast sources with the current mouse position
pub fn update_pick_source_positions(
    touches_input: Res<Touches>,
    mut cursor: EventReader<CursorMoved>,
    mut pick_source_query: Query<(
        &mut PickingCamera,
        Option<&mut UpdatePicks>,
        Option<&Camera>,
    )>,
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
) {
    for (mut pick_source, option_update_picks, option_camera) in &mut pick_source_query.iter_mut() {
        let (mut update_picks, cursor_latest) = match get_inputs(
            option_camera,
            option_update_picks,
            &mut cursor,
            &touches_input,
            &windows,
            &images,
        ) {
            Some(value) => value,
            None => continue,
        };
        match *update_picks {
            UpdatePicks::EveryFrame(cached_cursor_pos) => {
                match cursor_latest {
                    Some(cursor_moved) => {
                        pick_source.cast_method = RaycastMethod::Screenspace(cursor_moved);
                        *update_picks = UpdatePicks::EveryFrame(cursor_moved);
                    }
                    None => pick_source.cast_method = RaycastMethod::Screenspace(cached_cursor_pos),
                };
            }
            UpdatePicks::OnMouseEvent => match cursor_latest {
                Some(cursor_moved) => {
                    pick_source.cast_method = RaycastMethod::Screenspace(cursor_moved)
                }
                None => continue,
            },
        };
    }
}

fn get_inputs<'a>(
    option_camera: Option<&Camera>,
    option_update_picks: Option<Mut<'a, UpdatePicks>>,
    cursor: &mut EventReader<CursorMoved>,
    touches_input: &Res<Touches>,
    windows: &Res<Windows>,
    images: &Res<Assets<Image>>,
) -> Option<(Mut<'a, UpdatePicks>, Option<Vec2>)> {
    let camera = option_camera?;
    let update_picks = option_update_picks?;
    let height = camera.logical_target_size()?.y;
    let cursor_latest = match cursor.iter().last() {
        Some(cursor_moved) => match camera.target {
            RenderTarget::Window(window) => {
                if cursor_moved.id == window {
                    Some(cursor_moved.position)
                } else {
                    None
                }
            }
            RenderTarget::Image(ref image) => {
                let window = windows.get(cursor_moved.id);
                let image = images.get(image);
                match (window, image) {
                    (Some(window), Some(image)) => {
                        // Convert screen space to texture space
                        let width = window.physical_width() as f32;
                        let height = window.physical_height() as f32;
                        let x01 = cursor_moved.position.x / width;
                        let y01 = cursor_moved.position.y / height;
                        let size = image.size();
                        Some(Vec2::new(x01 * size.x, y01 * size.y))
                    }
                    _ => None,
                }
            }
        },
        None => touches_input
            .iter()
            .last()
            .map(|touch| Vec2::new(touch.position().x, height - touch.position().y)),
    };
    Some((update_picks, cursor_latest))
}
