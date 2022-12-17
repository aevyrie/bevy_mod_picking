//! A shader picking backend for `bevy_mod_picking`.
//!
//! # Shader Picking
//!
//! - Render to a texture, with dimensions set by DPI. Fuzzy picking should be set in "points" not
//!   pixels, so it scales with DPI properly
//! - Render all entities with a color defined by entity ID, note we can ignore the generation.
//! - Entity id is a u32, or 32 bits. RGBA is 32 bits per channel. Could pack entity id into a
//!   single channel, with room for distance data?
//!
//! # "Fuzzy" Picking
//!
//! Find closest entity
//!
//! # Impl
//!
//! group all cameras by `render_target`, note that pointers also correspond to render target sort
//! cameras by `priority` create a picking camera as a child of the camera, so it follows it. set
//! the target of the picking camera to be a texture
//!
//! Is the above just duplicating work already done in bevy_render?
//!
//! Instead, need to add a node to the render graph that takes the output from the vertex shader,
//! and runs the picking frag shader, writing to a texture.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::prelude::*;
use bevy_picking_core::backend::*;

/// Commonly used imports for the [`bevy_picking_shader`] crate.
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
