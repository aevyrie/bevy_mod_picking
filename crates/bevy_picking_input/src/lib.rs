use bevy::{
    ecs::schedule::ShouldRun, input::keyboard::KeyboardInput, prelude::*,
    render::camera::RenderTarget, window::WindowId,
};

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Cursor>()
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_touch)
                    .with_system(touch_pick_events),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_cursor)
                    .with_system(mouse_pick_events),
            );
    }
}

pub struct InputPluginSettings {
    update: UpdatePicks,
    use_cursor: bool,
    use_touch: bool,
}
impl Default for InputPluginSettings {
    fn default() -> Self {
        Self {
            update: Default::default(),
            use_cursor: true,
            use_touch: true,
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

fn run_if_cursor(settings: Res<InputPluginSettings>) -> ShouldRun {
    if settings.use_cursor {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

#[derive(Debug, Clone)]
pub enum UpdatePicks {
    EveryFrame,
    OnEvent,
}
impl Default for UpdatePicks {
    fn default() -> Self {
        UpdatePicks::EveryFrame
    }
}

#[derive(Debug, Clone)]
pub struct Cursor {
    target: RenderTarget,
    position: Vec2,
    clicked: bool,
    multiselect: bool,
}

pub struct InputState {
    multiselect: bool,
    clicked: bool,
}

pub fn default_picking_input(
    input: Res<InputState>,
    mouse: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
) {
    input.multiselect = keyboard.any_pressed([
        KeyCode::LControl,
        KeyCode::RControl,
        KeyCode::LShift,
        KeyCode::RShift,
    ]);
    input.clicked = mouse.pressed(MouseButton::Left)
}

/// Sends touch positions to be processed by the picking backend
pub fn touch_pick_events(
    input: Res<InputState>,
    settings: Res<InputPluginSettings>,
    touches: Res<Touches>,
    mut touch_events: EventReader<TouchInput>,
    mut pick_pos: EventWriter<Cursor>,
) {
    let mut cursor_positions = Vec::new();

    match settings.update {
        UpdatePicks::EveryFrame => {
            for touch in touches.iter() {
                cursor_positions.push(Cursor {
                    target: RenderTarget::Window(WindowId::primary()),
                    position: touch.position(),
                    clicked: true,
                    multiselect: input.multiselect,
                });
            }
        }
        UpdatePicks::OnEvent => {
            for event in touch_events.iter() {
                cursor_positions.push(Cursor {
                    target: RenderTarget::Window(WindowId::primary()),
                    position: event.position,
                    clicked: true,
                    multiselect: input.multiselect,
                })
            }
        }
    }

    pick_pos.send_batch(cursor_positions.into_iter());
}

/// Sends cursor positions to be processed by the picking backend
pub fn mouse_pick_events(
    input: Res<InputState>,
    settings: Res<InputPluginSettings>,
    windows: Res<Windows>,
    mut cursor_events: EventReader<CursorMoved>,
    mut pick_pos: EventWriter<Cursor>,
    mut click_events: EventReader<Click>,
) {
    let mut cursor_positions = Vec::new();
    match settings.update {
        UpdatePicks::EveryFrame => {
            for window in windows.iter() {
                if let Some(position) = window.cursor_position() {
                    let InputState {
                        clicked,
                        multiselect,
                    };
                    cursor_positions.push(Cursor {
                        target: RenderTarget::Window(window.id()),
                        position,
                        clicked,
                        multiselect,
                    });
                }
            }
        }
        UpdatePicks::OnEvent => {
            for event in cursor_events.iter() {
                cursor_positions.push(Cursor {
                    target: RenderTarget::Window(event.id),
                    position: event.position,
                    clicked: todo!(),
                    multiselect: todo!(),
                })
            }
        }
    }

    pick_pos.send_batch(cursor_positions.into_iter());
}
