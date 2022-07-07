use bevy::{app::PluginGroupBuilder, prelude::*};

pub use bevy_picking_core::*;
pub use bevy_picking_input::*;

#[cfg(feature = "rapier")]
pub use bevy_picking_rapier::*;

#[cfg(feature = "raycast")]
pub use bevy_picking_raycast::*;

pub struct DefaultPickingPlugins;
impl PluginGroup for DefaultPickingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(CorePlugin)
            .add(DefaultPointersPlugin)
            .add(InputPlugin)
            .add(InteractionPlugin);

        #[cfg(feature = "raycast")]
        group.add(RaycastPlugin);
        #[cfg(feature = "rapier")]
        group.add(RapierPlugin);

        HighlightingPlugins.build(group);
    }
}
