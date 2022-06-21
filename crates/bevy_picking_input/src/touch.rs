use bevy::{prelude::*, render::camera::RenderTarget, utils::HashMap, window::WindowId};
use bevy_picking_core::picking::{
    cursor::{Cursor, CursorId},
    CursorBundle,
};

/// Sends touch positions to be processed by the picking backend
pub fn touch_pick_events(
    mut commands: Commands,
    touches: Res<Touches>,
    mut cursor_query: Query<(&CursorId, &mut Cursor)>,
) {
    let mut new_cursor_map = HashMap::new();
    for touch in touches.iter() {
        let id = CursorId::Touch(touch.id());
        new_cursor_map.insert(
            id,
            Cursor {
                enabled: true,
                clicked: false,
                target: RenderTarget::Window(WindowId::primary()),
                position: touch.position(),
            },
        );
    }
    // Update existing cursors
    for (id, mut cursor) in cursor_query.iter_mut() {
        if !id.is_touch() {
            continue;
        }
        match new_cursor_map.remove(&id) {
            Some(new_cursor) => *cursor = new_cursor,
            None => cursor.enabled = false,
        }
    }
    // Spawn new cursors if needed
    for (id, cursor) in new_cursor_map.drain() {
        commands.spawn_bundle(CursorBundle::new(id, cursor));
    }
}
