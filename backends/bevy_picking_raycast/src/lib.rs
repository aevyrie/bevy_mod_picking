//! A raycasting backend for `bevy_mod_picking` that uses `bevy_mod_raycast` for raycasting.

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
#[derive(Resource, Default)]
pub struct RaycastBackendSettings {
    /// When set to `true` raycasting will only happen between cameras marked with
    /// [`RaycastPickCamera`] and entities marked with [`RaycastPickTarget`]. Off by default.
    pub require_markers: bool,
}

/// This unit struct is used to tag the generic ray casting types [`RaycastMesh`] and
/// [`RaycastSource`].
#[derive(Reflect, Clone)]
pub struct RaycastPickingSet;

/// Marks an entity that should be pickable with [`bevy_mod_raycast`] ray casts. Only needed if
/// [`RaycastBackendSettings::require_markers`] is set to true.
pub type RaycastPickTarget = RaycastMesh<RaycastPickingSet>;

/// Marks a camera that should be used for picking with [`bevy_mod_raycast`]. Only needed if
/// [`RaycastBackendSettings::require_markers`] is set to true.
#[derive(Debug, Default, Clone, Component, Reflect)]
pub struct RaycastPickCamera;

/// Adds the raycasting picking backend to your app.
#[derive(Clone)]
pub struct RaycastBackend;
impl Plugin for RaycastBackend {
    fn build(&self, app: &mut App) {
        app.init_resource::<RaycastBackendSettings>()
            .add_systems(PreUpdate, update_hits.in_set(PickSet::Backend));
    }
}

/// Builds rays and updates raycasting [`RaycastPickCamera`]s from [`PointerLocation`]s.
pub fn update_hits(
    pointers: Query<(&PointerId, &PointerLocation)>,
    primary_window_entity: Query<Entity, With<PrimaryWindow>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    picking_cameras: Query<(
        Entity,
        &Camera,
        &GlobalTransform,
        Option<&RaycastPickCamera>,
        Option<&RenderLayers>,
    )>,
    pickables: Query<&Pickable>,
    marked_targets: Query<&RaycastPickTarget>,
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
