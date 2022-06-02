use bevy::{
    prelude::*,
    render::camera::{Camera, RenderTarget},
    window::WindowId,
};
use bevy_picking_core::PickingInput;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PickingPosition>()
            .add_system(pick_position_events);
    }
}

pub struct InputPluginSettings {
    update: UpdatePicks,
}

#[derive(Debug, Clone)]
pub enum UpdatePicks {
    EveryFrame,
    OnMouseEvent,
}
impl Default for UpdatePicks {
    fn default() -> Self {
        UpdatePicks::EveryFrame
    }
}

#[derive(Debug, Clone)]
pub struct PickingPosition {
    window: WindowId,
    position: Vec2,
}

pub fn set_picking_input(input: ResMut<PickingInput>) {}

pub fn pick_position_events(
    input: ResMut<PickingInput>,
    touches: Res<Touches>,
    mut settings: ResMut<InputPluginSettings>,
    mut cursor_events: EventReader<CursorMoved>,
    mut pick_pos: EventWriter<PickingPosition>,
    /// The last known picking position
    mut position_cache: Local<Vec<PickingPosition>>,
) {
    let current = Vec::new();

    for touch in touches.iter() {
        current.push(PickingPosition {
            window: WindowId::primary(),
            position: touch.position(),
        });
    }

    for cursor in cursor_events.iter() {
        current.push(PickingPosition {
            window: cursor.id,
            position: cursor.position,
        });
    }

    if current.is_empty() {
        current = position_cache;
    } else {
        position_cache = current;
    }

    let result = if let UpdatePicks::EveryFrame = settings.update {
        if current.is_empty() {
            latest
        } else {
            latest = current;
            current
        }
    } else {
        current
    };

    pick_pos.send_batch(result.into_iter());
}
