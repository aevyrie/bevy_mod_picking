//! A raycasting backend for `bevy_mod_picking` that uses `xpbd` for raycasting.
//!
//! # Usage
//!
//! Pointers will automatically shoot rays into the xpbd scene and pick entities.
//!
//! To ignore an entity, you can add [`Pickable::IGNORE`] to it, and it will be ignored during
//! raycasting.
//!
//! For fine-grained control, see the [`XpbdBackendSettings::require_markers`] setting.
//!
//! ## Limitations
//!
//! Because raycasting is expensive, only the closest intersection will be reported. This means that
//! unlike some UI, you cannot hover multiple xpbd objects with a single pointer by configuring the
//! [`Pickable`] component to not block lower elements but still emit events. As mentioned above,
//! all that is supported is completely ignoring an entity with [`Pickable::IGNORE`].
//!
//! This is probably not a meaningful limitation, as the feature is usually only used in UI where
//! you might want a pointer to be able to pick multiple elements that are on top of each other. If
//! are trying to build a UI out of xpbd entities, beware, I suppose.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_reflect::{std_traits::ReflectDefault, Reflect};
use bevy_render::{prelude::*, view::RenderLayers};

use bevy_picking_core::backend::prelude::*;
use bevy_xpbd_3d::prelude::*;

// Re-export for users who want this
pub use bevy_xpbd_3d;

/// Commonly used imports.
pub mod prelude {
    pub use crate::{XpbdBackend, XpbdBackendSettings, XpbdPickable};
}

/// Adds the `xpbd_3d` raycasting picking backend to your app.
#[derive(Clone)]
pub struct XpbdBackend;
impl Plugin for XpbdBackend {
    fn build(&self, app: &mut App) {
        app.init_resource::<XpbdBackendSettings>()
            .add_systems(PreUpdate, update_hits.in_set(PickSet::Backend))
            .register_type::<XpbdBackendSettings>()
            .register_type::<XpbdPickable>();
    }
}

/// Runtime settings for the [`XpbdBackend`].
#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct XpbdBackendSettings {
    /// When set to `true` raycasting will only happen between cameras and entities marked with
    /// [`XpbdPickable`]. Off by default. This setting is provided to give you fine-grained
    /// control over which cameras and entities should be used by the xpbd backend at runtime.
    pub require_markers: bool,
}

/// Optional. Marks cameras and target entities that should be used in the xpbd picking backend.
/// Only needed if [`XpbdBackendSettings::require_markers`] is set to true.
#[derive(Debug, Clone, Default, Component, Reflect)]
#[reflect(Component, Default)]
pub struct XpbdPickable;

/// Raycasts into the scene using [`XpbdBackendSettings`] and [`PointerLocation`]s, then outputs
/// [`PointerHits`].
pub fn update_hits(
    picking_cameras: Query<(&Camera, Option<&XpbdPickable>, Option<&RenderLayers>)>,
    ray_map: Res<RayMap>,
    pickables: Query<&Pickable>,
    marked_targets: Query<&XpbdPickable>,
    layers: Query<&RenderLayers>,
    backend_settings: Res<XpbdBackendSettings>,
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

        let cam_layers = cam_layers.copied().unwrap_or_default();

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
                    let entity_layers = layers.get(entity).copied().unwrap_or_default();
                    let render_layers_match = cam_layers.intersects(&entity_layers);

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
