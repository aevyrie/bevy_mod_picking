//! A raycasting backend for `bevy_mod_picking` that uses `Avian` for raycasting.
//!
//! # Usage
//!
//! Pointers will automatically shoot rays into the Avian scene and pick entities.
//!
//! To ignore an entity, you can add [`Pickable::IGNORE`] to it, and it will be ignored during
//! raycasting.
//!
//! For fine-grained control, see the [`AvianBackendSettings::require_markers`] setting.
//!
//! ## Limitations
//!
//! Because raycasting is expensive, only the closest intersection will be reported. This means that
//! unlike some UI, you cannot hover multiple Avian objects with a single pointer by configuring the
//! [`Pickable`] component to not block lower elements but still emit events. As mentioned above,
//! all that is supported is completely ignoring an entity with [`Pickable::IGNORE`].
//!
//! This is probably not a meaningful limitation, as the feature is usually only used in UI where
//! you might want a pointer to be able to pick multiple elements that are on top of each other. If
//! are trying to build a UI out of Avian entities, beware, I suppose.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_reflect::{std_traits::ReflectDefault, Reflect};
use bevy_render::{prelude::*, view::RenderLayers};

use avian3d::prelude::*;
use bevy_picking_core::backend::prelude::*;

// Re-export for users who want this
pub use avian3d;

/// Commonly used imports.
pub mod prelude {
    pub use crate::{AvianBackend, AvianBackendSettings, AvianPickable};
}

/// Adds the `avian3d` raycasting picking backend to your app.
#[derive(Clone)]
pub struct AvianBackend;
impl Plugin for AvianBackend {
    fn build(&self, app: &mut App) {
        app.init_resource::<AvianBackendSettings>()
            .add_systems(PreUpdate, update_hits.in_set(PickSet::Backend))
            .register_type::<AvianBackendSettings>()
            .register_type::<AvianPickable>();
    }
}

/// Runtime settings for the [`AvianBackend`].
#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct AvianBackendSettings {
    /// When set to `true` raycasting will only happen between cameras and entities marked with
    /// [`AvianPickable`]. Off by default. This setting is provided to give you fine-grained
    /// control over which cameras and entities should be used by the avian backend at runtime.
    pub require_markers: bool,
}

/// Optional. Marks cameras and target entities that should be used in the avian picking backend.
/// Only needed if [`AvianBackendSettings::require_markers`] is set to true.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct AvianPickable;

/// Raycasts into the scene using [`AvianBackendSettings`] and [`PointerLocation`]s, then outputs
/// [`PointerHits`].
pub fn update_hits(
    picking_cameras: Query<(&Camera, Option<&AvianPickable>, Option<&RenderLayers>)>,
    ray_map: Res<RayMap>,
    pickables: Query<&Pickable>,
    marked_targets: Query<&AvianPickable>,
    layers: Query<&RenderLayers>,
    backend_settings: Res<AvianBackendSettings>,
    spatial_query: Option<Res<SpatialQueryPipeline>>,
    mut output_events: EventWriter<PointerHits>,
) {
    let Some(spatial_query) = spatial_query else {
        return;
    };

    for (&ray_id, &ray) in ray_map.map().iter() {
        let Ok((camera, cam_pickable, cam_layers)) = picking_cameras.get(ray_id.camera) else {
            continue;
        };
        if backend_settings.require_markers && cam_pickable.is_none() || !camera.is_active {
            continue;
        }

        let cam_layers = cam_layers.unwrap_or_default();

        if let Some((entity, hit_data)) = spatial_query
            .cast_ray_predicate(
                ray.origin,
                ray.direction,
                f32::MAX,
                true,
                SpatialQueryFilter::default(),
                &|entity| {
                    let marker_requirement =
                        !backend_settings.require_markers || marked_targets.get(entity).is_ok();

                    // Other entities missing render layers are on the default layer 0
                    let entity_layers = layers.get(entity).unwrap_or_default();
                    let render_layers_match = cam_layers.intersects(entity_layers);

                    let is_pickable = pickables
                        .get(entity)
                        .map(|p| *p != Pickable::IGNORE)
                        .unwrap_or(true);

                    marker_requirement && render_layers_match && is_pickable
                },
            )
            .map(|ray_hit_data| {
                let hit_data = HitData::new(
                    ray_id.camera,
                    ray_hit_data.time_of_impact,
                    Some(ray.origin + (ray.direction * ray_hit_data.time_of_impact)),
                    Some(ray_hit_data.normal),
                );
                (ray_hit_data.entity, hit_data)
            })
        {
            output_events.send(PointerHits::new(
                ray_id.pointer,
                vec![(entity, hit_data)],
                camera.order as f32,
            ));
        }
    }
}
