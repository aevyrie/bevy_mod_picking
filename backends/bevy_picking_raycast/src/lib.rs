//! A raycasting backend for `bevy_mod_picking` that uses `bevy_mod_raycast` for raycasting.
//!
//! # Usage
//!
//! If a pointer passes through this camera's render target, it will automatically shoot rays into
//! the scene and will be able to pick things.
//!
//! To ignore an entity, you can add [`Pickable::IGNORE`] to it, and it will be ignored during
//! raycasting.
//!
//! For fine-grained control, see the [`RaycastBackendSettings::require_markers`] setting.
//!

#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![deny(missing_docs)]

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use bevy_render::{prelude::*, view::RenderLayers};
use bevy_transform::prelude::*;
use bevy_window::{PrimaryWindow, Window};

use bevy_mod_raycast::prelude::*;
use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_raycast`](crate) crate.
pub mod prelude {
    pub use crate::RaycastBackend;
}

/// Runtime settings for the [`RaycastBackend`].
#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct RaycastBackendSettings {
    /// When set to `true` raycasting will only happen between cameras and entities marked with
    /// [`RaycastPickable`]. Off by default. This setting is provided to give you fine-grained
    /// control over which cameras and entities should be used by the rapier backend at runtime.
    pub require_markers: bool,
}

/// Optional. Marks cameras and target entities that should be used in the raycast picking backend.
/// Only needed if [`RaycastBackendSettings::require_markers`] is set to true.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct RaycastPickable;

/// Adds the raycasting picking backend to your app.
#[derive(Clone)]
pub struct RaycastBackend;
impl Plugin for RaycastBackend {
    fn build(&self, app: &mut App) {
        app.init_resource::<RaycastBackendSettings>()
            .add_systems(PreUpdate, update_hits.in_set(PickSet::Backend));
    }
}

/// Raycasts into the scene using [`RaycastBackendSettings`] and [`PointerLocation`]s, then outputs
/// [`PointerHits`].
pub fn update_hits(
    pointers: Query<(&PointerId, &PointerLocation)>,
    primary_window_entity: Query<Entity, With<PrimaryWindow>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    picking_cameras: Query<(
        Entity,
        &Camera,
        &GlobalTransform,
        Option<&RaycastPickable>,
        Option<&RenderLayers>,
    )>,
    pickables: Query<&Pickable>,
    marked_targets: Query<&RaycastPickable>,
    layers: Query<&RenderLayers>,
    backend_settings: Res<RaycastBackendSettings>,
    mut raycast: Raycast,
    mut output_events: EventWriter<PointerHits>,
) {
    for (pointer_id, pointer_location) in &pointers {
        let pointer_location = match pointer_location.location() {
            Some(l) => l,
            None => continue,
        };
        for (cam_entity, camera, ray, cam_layers) in picking_cameras
            .iter()
            .filter(|(_, camera, ..)| {
                camera.is_active && pointer_location.is_in_viewport(camera, &primary_window_entity)
            })
            .filter(|(.., marker, _)| marker.is_some() || !backend_settings.require_markers)
            .filter_map(|(entity, camera, transform, _, layers)| {
                Ray3d::from_screenspace(
                    pointer_location.position,
                    camera,
                    transform,
                    primary_window.single(),
                )
                .map(|ray| (entity, camera, ray, layers))
            })
        {
            let settings = bevy_mod_raycast::system_param::RaycastSettings {
                visibility: RaycastVisibility::MustBeVisibleAndInView,
                filter: &|entity| {
                    let marker_requirement =
                        !backend_settings.require_markers || marked_targets.get(entity).is_ok();
                    let render_layers_match = match (cam_layers, layers.get(entity)) {
                        (Some(cam_layers), Ok(entity_layers)) => {
                            cam_layers.intersects(entity_layers)
                        }
                        _ => true, // If either `RenderLayers` components is not present, ignore.
                    };
                    marker_requirement && render_layers_match
                },
                early_exit_test: &|entity_hit| {
                    pickables
                        .get(entity_hit)
                        .is_ok_and(|pickable| pickable.should_block_lower)
                },
            };
            let picks = raycast
                .cast_ray(ray, &settings)
                .iter()
                .map(|(entity, hit)| {
                    let hit_data = HitData::new(
                        cam_entity,
                        hit.distance(),
                        Some(hit.position()),
                        Some(hit.normal()),
                    );
                    (*entity, hit_data)
                })
                .collect::<Vec<_>>();
            let order = camera.order as f32;
            if !picks.is_empty() {
                output_events.send(PointerHits::new(*pointer_id, picks, order));
            }
        }
    }
}
