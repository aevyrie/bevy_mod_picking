use bevy::{
    ecs::schedule::ShouldRun, prelude::*, render::camera::RenderTarget, utils::HashMap,
    window::WindowId,
};
use bevy_picking_core::picking::{
    cursor::{Cursor, CursorId, MultiSelect},
    CursorBundle,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
enum Set {
    Input,
    Touch,
    Mouse,
}

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .label(Set::Input)
                .with_run_criteria(run_if_default_inputs)
                .with_system(default_picking_inputs),
        )
        .add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .label(Set::Touch)
                .with_run_criteria(run_if_touch)
                .with_system(touch_pick_events)
                .after(Set::Input),
        )
        .add_system_set_to_stage(
            CoreStage::First,
            SystemSet::new()
                .label(Set::Mouse)
                .with_run_criteria(run_if_mouse)
                .with_system(mouse_pick_events)
                .after(Set::Input),
        );
    }
}

pub struct InputPluginSettings {
    mode: UpdateMode,
    use_mouse: bool,
    use_touch: bool,
    use_default_buttons: bool,
}
impl Default for InputPluginSettings {
    fn default() -> Self {
        Self {
            mode: Default::default(),
            use_mouse: true,
            use_touch: true,
            use_default_buttons: true,
        }
    }
}

fn run_if_touch(settings: Res<InputPluginSettings>) -> ShouldRun {
    if settings.use_touch {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn run_if_mouse(settings: Res<InputPluginSettings>) -> ShouldRun {
    if settings.use_mouse {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn run_if_default_inputs(settings: Res<InputPluginSettings>) -> ShouldRun {
    if settings.use_default_buttons {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

#[derive(Debug, Clone)]
pub enum UpdateMode {
    EveryFrame,
    OnEvent,
}
impl Default for UpdateMode {
    fn default() -> Self {
        UpdateMode::EveryFrame
    }
}

/// Unsurprising default picking inputs
pub fn default_picking_inputs(
    mut multiselect: ResMut<MultiSelect>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut cursor_query: Query<(&CursorId, &mut Cursor)>,
    touches: Res<Touches>,
) {
    multiselect.active = keyboard.any_pressed([
        KeyCode::LControl,
        KeyCode::RControl,
        KeyCode::LShift,
        KeyCode::RShift,
    ]);

    for (&id, mut cursor) in cursor_query.iter_mut() {
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
        match new_cursor_map.remove(&id) {
            Some(new_cursor) => *cursor = new_cursor,
            None => cursor.enabled = false,
        }
    }

    // Spawn  new cursors if needed
    for (id, cursor) in new_cursor_map.drain() {
        commands.spawn_bundle(CursorBundle::new(id, cursor));
    }
}

/// Sends cursor positions to be processed by the picking backend
pub fn mouse_pick_events(
    mut commands: Commands,
    settings: Res<InputPluginSettings>,
    windows: Res<Windows>,
    cursor_move: EventReader<CursorMoved>,
    cursor_leave: EventReader<CursorLeft>,
    mut cursor_query: Query<(&CursorId, &mut Cursor)>,
) {
    let mut try_cursor = None;

    for window in windows.iter() {
        if let Some(position) = window.cursor_position() {
            try_cursor = Some(Cursor {
                enabled: true,
                clicked: false,
                target: RenderTarget::Window(window.id()),
                position,
            });
        }
    }

    if let UpdateMode::OnEvent = settings.mode {
        if cursor_move.is_empty() && cursor_leave.is_empty() {
            return;
        }
    }

    // Update existing cursors
    if let Some(new_cursor) = try_cursor.take() {
        for (&id, mut cursor) in cursor_query.iter_mut() {
            if id == CursorId::Mouse {
                *cursor = new_cursor.clone();
            }
        }
    }

    // Spawn  new cursors if needed
    if let Some(cursor) = try_cursor {
        commands.spawn_bundle(CursorBundle::new(CursorId::Mouse, cursor));
    }
}
