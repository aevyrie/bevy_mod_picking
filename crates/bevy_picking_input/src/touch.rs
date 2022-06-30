use bevy::{prelude::*, render::camera::RenderTarget, utils::HashMap, window::WindowId};
use bevy_picking_core::{
    input::{CursorClick, CursorId, CursorLocation, Location},
    CursorBundle,
};

/// Sends touch positions to be processed by the picking backend
pub fn touch_pick_events(
    mut commands: Commands,
    touches: Res<Touches>,
    mut cursor_query: Query<(&CursorId, &mut CursorLocation, &mut CursorClick)>,
) {
    let mut new_cursor_map = HashMap::new();
    for touch in touches.iter() {
        let id = CursorId::Touch(touch.id());
        new_cursor_map.insert(
            id,
            (
                CursorLocation {
                    location: Some(Location {
                        target: RenderTarget::Window(WindowId::primary()),
                        position: touch.position(),
                    }),
                },
                CursorClick { is_clicked: true },
            ),
        );
    }
    // Update existing cursor entities
    for (id, mut cursor, mut click) in cursor_query.iter_mut() {
        if !id.is_touch() {
            continue;
        }
        match new_cursor_map.remove(&id) {
            Some(new_cursor) => {
                if (cursor.as_ref(), click.as_ref()) != (&new_cursor.0, &new_cursor.1) {
                    (*cursor, *click) = new_cursor;
                }
            }
            None => {
                if cursor.as_ref().location.is_some() {
                    cursor.location = None
                }
                if click.as_ref().is_clicked {
                    click.is_clicked = false;
                }
            }
        }
    }
    // Spawn new cursor entities if needed
    for (id, cursor) in new_cursor_map.drain() {
        commands.spawn_bundle(CursorBundle::new(id, cursor.0, cursor.1));
    }
}
