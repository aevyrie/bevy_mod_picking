#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![warn(missing_docs)]

use bevy::prelude::*;

pub struct RapierPlugin;
impl Plugin for RapierPlugin {
    fn build(&self, _app: &mut App) {}
}
