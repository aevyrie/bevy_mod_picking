//! `bevy_picking_input` is a thin layer that provides unsurprising default inputs to `bevy_picking
//! core`. The included systems are responsible for sending  mouse and touch inputs to their
//! respective `Pointer`s.
//!
//! Because this resides in its own crate, it's easy to omit it, and provide your own inputs as
//! needed. Because `Pointer`s aren't coupled to the underlying input hardware, you can easily mock
//! inputs, and allow users full accessibility to map whatever inputs they need to pointer input.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::{ecs::schedule::ShouldRun, prelude::*};
use bevy_picking_core::{
    pointer::{InputMove, InputPress, PointerId},
    PickStage, PointerBundle,
};

pub mod debug;
pub mod mouse;
pub mod touch;

/// Adds mouse and touch inputs for picking pointers to your app.
pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputPluginSettings>()
            .add_system_set_to_stage(
                CoreStage::First,
                SystemSet::new()
                    .label(PickStage::Input)
                    .with_system(
                        touch::touch_pick_events
                            .with_run_criteria(run_if_touch)
                            .before(spawn_new_pointers),
                    )
                    .with_system(
                        mouse::mouse_pick_events
                            .with_run_criteria(run_if_mouse)
                            .before(spawn_new_pointers),
                    )
                    .with_system(spawn_new_pointers),
            );
    }
}

/// Settings for the input plugin to allow enabling or disabling mouse or touch inputs at runtime.
pub struct InputPluginSettings {
    run_mouse: bool,
    run_touch: bool,
}
impl Default for InputPluginSettings {
    fn default() -> Self {
        Self {
            run_mouse: true,
            run_touch: true,
        }
    }
}

/// Spawn new pointers when an event comes in for a pointer that doesn't exist yet.
pub fn spawn_new_pointers(
    mut commands: Commands,
    pointers: Query<&PointerId>,
    mut moves: EventReader<InputMove>,
    mut press: EventReader<InputPress>,
) {
    moves
        .iter()
        .map(|e| e.id())
        .chain(press.iter().map(|e| e.id()))
        .filter(|id| !pointers.iter().any(|p| p == id))
        .for_each(|id| {
            commands.spawn_bundle(PointerBundle::new(id));
        });
}

fn run_if_touch(settings: Res<InputPluginSettings>) -> ShouldRun {
    settings.run_touch.should_run()
}
fn run_if_mouse(settings: Res<InputPluginSettings>) -> ShouldRun {
    settings.run_mouse.should_run()
}

/// Simple trait used to convert a boolean to a run criteria.
trait IntoShouldRun {
    /// Converts `self` into [`ShouldRun`].
    fn should_run(&self) -> ShouldRun;
}
impl IntoShouldRun for bool {
    fn should_run(&self) -> ShouldRun {
        if *self {
            ShouldRun::Yes
        } else {
            ShouldRun::No
        }
    }
}
