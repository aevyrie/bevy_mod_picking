use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_picking_core::{input::PointerMultiselect, IntoShouldRun, PickStage};

pub mod mouse;
pub mod touch;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputPluginSettings>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .label(PickStage::Input)
                    .with_system(touch::touch_pick_events.with_run_criteria(run_if_touch))
                    .with_system(mouse::mouse_pick_events.with_run_criteria(run_if_mouse))
                    .with_system(multiselect_events.with_run_criteria(run_if_multiselect)),
            );
    }
}

pub struct InputPluginSettings {
    run_mouse: bool,
    run_touch: bool,
    run_multiselect: bool,
}
impl Default for InputPluginSettings {
    fn default() -> Self {
        Self {
            run_mouse: true,
            run_touch: true,
            run_multiselect: true,
        }
    }
}

/// Unsurprising default multiselect inputs
pub fn multiselect_events(
    keyboard: Res<Input<KeyCode>>,
    mut pointer_query: Query<&mut PointerMultiselect>,
) {
    let is_multiselect_pressed = keyboard.any_pressed([
        KeyCode::LControl,
        KeyCode::RControl,
        KeyCode::LShift,
        KeyCode::RShift,
    ]);

    for mut multiselect in pointer_query.iter_mut() {
        multiselect.is_pressed = is_multiselect_pressed;
    }
}

fn run_if_touch(settings: Res<InputPluginSettings>) -> ShouldRun {
    settings.run_touch.should_run()
}
fn run_if_mouse(settings: Res<InputPluginSettings>) -> ShouldRun {
    settings.run_mouse.should_run()
}
fn run_if_multiselect(settings: Res<InputPluginSettings>) -> ShouldRun {
    settings.run_multiselect.should_run()
}
