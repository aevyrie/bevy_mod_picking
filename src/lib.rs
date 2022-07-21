use bevy::{app::PluginGroupBuilder, prelude::*, ui::FocusPolicy};
use bevy_picking_core::focus::PickLayer;

pub use bevy_picking_core::*;
pub use bevy_picking_input::*;
// Optional
#[cfg(feature = "highlight")]
pub use bevy_picking_highlight as highlight;
#[cfg(feature = "selection")]
pub use bevy_picking_selection as selection;
// Backends
#[cfg(feature = "rapier")]
pub use bevy_picking_rapier as rapier;
#[cfg(feature = "raycast")]
pub use bevy_picking_raycast as raycast;
#[cfg(feature = "pick_shader")]
pub use bevy_picking_shader as shader;

pub struct DefaultPickingPlugins;
impl PluginGroup for DefaultPickingPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(CorePlugin)
            .add(DefaultPointersPlugin)
            .add(InputPlugin)
            .add(InteractionPlugin);

        // Optional
        #[cfg(feature = "selection")]
        group.add(selection::SelectionPlugin);
        #[cfg(feature = "highlight")]
        highlight::HighlightingPlugins.build(group);

        // Backends
        #[cfg(feature = "raycast")]
        group.add(raycast::RaycastPlugin);
        #[cfg(feature = "rapier")]
        group.add(rapier::RapierPlugin);
        #[cfg(feature = "pick_shader")]
        group.add(shader::ShaderPlugin);
    }
}

/// Makes an entity pickable.
#[derive(Bundle, Default)]
pub struct PickableBundle {
    pub pick_layer: PickLayer,
    pub interaction: Interaction,
    pub focus_policy: FocusPolicy,
    #[cfg(feature = "selection")]
    pub selection: bevy_picking_selection::PickSelection,
    #[cfg(feature = "highlight")]
    pub highlight: bevy_picking_highlight::PickHighlight,
}

/// Components needed for a pointer
#[derive(Bundle)]
pub struct PointerBundle {
    pub id: PointerId,
    pub location: input::PointerPosition,
    pub click: input::PointerPress,
    pub interaction: output::PointerInteraction,
    #[cfg(feature = "selection")]
    pub multi_select: bevy_picking_selection::PointerMultiselect,
}
impl PointerBundle {
    pub fn new(id: PointerId) -> Self {
        PointerBundle {
            id,
            location: input::PointerPosition::default(),
            click: input::PointerPress::default(),
            interaction: output::PointerInteraction::default(),
            #[cfg(feature = "selection")]
            multi_select: selection::PointerMultiselect::default(),
        }
    }
}

pub struct DefaultPointersPlugin;
impl Plugin for DefaultPointersPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_default_pointers);
    }
}

pub fn add_default_pointers(mut commands: Commands) {
    commands.spawn_bundle(PointerBundle::new(PointerId::Mouse));
    // Windows supports up to 20 touch + 10 writing
    for i in 0..30 {
        commands.spawn_bundle(PointerBundle::new(PointerId::Touch(i)));
    }
}
