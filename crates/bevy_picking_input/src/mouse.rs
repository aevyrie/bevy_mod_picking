use bevy::{prelude::*, render::camera::RenderTarget};
use bevy_picking_core::{
    input::{CursorClick, CursorId, CursorInput},
    CursorBundle,
};

use crate::{InputPluginSettings, UpdateMode};

/// Updates [`CursorInput`]s to be processed by the picking backend
pub fn mouse_pick_events(
    mut commands: Commands,
    settings: Res<InputPluginSettings>,
    windows: Res<Windows>,
    cursor_move: EventReader<CursorMoved>,
    cursor_leave: EventReader<CursorLeft>,
    mut cursor_query: Query<(&CursorId, &mut CursorInput)>,
) {
    if matches!(settings.mode, UpdateMode::OnEvent)
        && cursor_move.is_empty()
        && cursor_leave.is_empty()
    {
        return;
    }
    let try_cursor = get_cursor_position(windows);
    update_cursor(&mut commands, try_cursor, &mut cursor_query);
}

fn get_cursor_position(windows: Res<Windows>) -> Option<(Vec2, RenderTarget)> {
    for window in windows.iter() {
        if let Some(position) = window.cursor_position() {
            return Some((position, RenderTarget::Window(window.id())));
        }
    }
    None
}

fn update_cursor(
    commands: &mut Commands,
    try_cursor: Option<(Vec2, RenderTarget)>,
    cursor_query: &mut Query<(&CursorId, &mut CursorInput)>,
) {
    if let Some((position, target)) = try_cursor {
        for (&id, mut cursor) in cursor_query.iter_mut() {
            if !id.is_mouse() {
                continue;
            }
            if cursor.as_ref().position != position || cursor.as_ref().target != target {
                cursor.enabled = true;
                cursor.position = position;
                cursor.target = target;
            }
            return;
        }
        commands.spawn_bundle(CursorBundle::new(
            CursorId::Mouse,
            CursorInput {
                enabled: true,
                target,
                position,
                multiselect: false,
            },
            CursorClick { is_clicked: false },
        ));
    } else {
        for (&id, mut cursor) in cursor_query.iter_mut() {
            if !id.is_mouse() {
                continue;
            }
            cursor.enabled = false;
            return;
        }
    }
}
