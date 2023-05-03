//! A shader picking backend for `bevy_mod_picking`.
//!
//! STUB

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::prelude::*;
use bevy_picking_core::backend::*;

/// Commonly used imports.
pub mod prelude {
    // pub use crate::;
}

/// Adds support for shader picking to `bevy_mod_picking`.
#[derive(Clone)]
pub struct ShaderBackend;
impl PickingBackend for ShaderBackend {}
impl Plugin for ShaderBackend {
    fn build(&self, _app: &mut App) {}
}
