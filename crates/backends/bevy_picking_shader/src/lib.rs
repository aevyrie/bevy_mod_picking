//! A shader picking backend for `bevy_mod_picking`.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::prelude::*;

/// Adds support for shader picking to `bevy_mod_picking`.
pub struct ShaderPickingPlugin;
impl Plugin for ShaderPickingPlugin {
    fn build(&self, _app: &mut App) {}
}
