use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_picking_core::{IntoShouldRun, PickStage};

pub mod inputs;
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
                    .with_system(
                        inputs::default_picking_inputs
                            .with_run_criteria(run_if_default_inputs)
                            .after(touch::touch_pick_events)
                            .after(mouse::mouse_pick_events),
                    ),
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

fn run_if_touch(settings: Res<InputPluginSettings>) -> ShouldRun {
    settings.use_touch.should_run()
}
fn run_if_mouse(settings: Res<InputPluginSettings>) -> ShouldRun {
    settings.use_mouse.should_run()
}
fn run_if_default_inputs(settings: Res<InputPluginSettings>) -> ShouldRun {
    settings.use_default_buttons.should_run()
}
