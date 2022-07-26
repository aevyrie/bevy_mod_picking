//! A raycasting backend for `bevy_mod_picking` that uses `rapier` for raycasting.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::prelude::*;

/// Adds the `rapier` raycasting picking backend to your app.
pub struct RapierPlugin;
impl Plugin for RapierPlugin {
    fn build(&self, _app: &mut App) {}
}
