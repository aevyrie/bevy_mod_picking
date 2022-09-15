//! A shader picking backend for `bevy_mod_picking`.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::prelude::*;

/// Commonly used imports for the [`bevy_picking_shader`] crate.
pub mod prelude {
    // pub use crate::;
}

/// Adds support for shader picking to `bevy_mod_picking`.
pub struct ShaderPlugin;
impl Plugin for ShaderPlugin {
    fn build(&self, _app: &mut App) {
        unimplemented!();
    }
}
