//! A raycasting backend for [`bevy_sprite`].

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use std::cmp::Ordering;

use bevy_app::prelude::*;
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_render::prelude::*;
use bevy_sprite::{Sprite, TextureAtlas, TextureAtlasSprite};
use bevy_transform::prelude::*;
use bevy_window::PrimaryWindow;

use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_sprite`](crate) crate.
pub mod prelude {
    pub use crate::SpriteBackend;
}

/// Adds picking support for [`bevy_sprite`].
#[derive(Clone)]
pub struct SpriteBackend;

impl Plugin for SpriteBackend {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, sprite_picking.in_set(PickSet::Backend));
    }
}

/// Checks if any sprite entities are under each pointer
pub fn sprite_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    texture_atlas: Res<Assets<TextureAtlas>>,
    sprite_query: Query<
        (
            Entity,
            (Option<&Sprite>, Option<&TextureAtlasSprite>),
            (Option<&Handle<Image>>, Option<&Handle<TextureAtlas>>),
            &GlobalTransform,
            &ComputedVisibility,
            Option<&Pickable>,
        ),
        Or<(With<Sprite>, With<TextureAtlasSprite>)>,
    >,
    mut output: EventWriter<PointerHits>,
) {
    let mut sorted_sprites: Vec<_> = sprite_query.iter().collect();
    sorted_sprites.sort_by(|a, b| {
        (b.3.translation().z)
            .partial_cmp(&a.3.translation().z)
            .unwrap_or(Ordering::Equal)
    });

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let mut blocked = false;
        let Some((cam_entity, camera, cam_transform)) = cameras
            .iter()
            .filter(|(_, camera, _)| camera.is_active)
            .find(|(_, camera, _)| {
                camera
                    .target
                    .normalize(Some(primary_window.single()))
                    .unwrap()
                    == location.target
            })
        else {
            continue;
        };

        let Some(cursor_pos_world) = camera.viewport_to_world_2d(cam_transform, location.position)
        else {
            continue;
        };

        let picks: Vec<(Entity, HitData)> = sorted_sprites
            .iter()
            .copied()
            .filter_map(
                |(entity, sprite, image, sprite_transform, visibility, sprite_focus)| {
                    if blocked || !visibility.is_visible() {
                        return None;
                    }
                    let custom_size = if sprite.0.is_some() {
                        sprite.0.unwrap().custom_size
                    } else {
                        sprite.1.unwrap().custom_size
                    };
                    let anchor = if sprite.0.is_some() {
                        &sprite.0.unwrap().anchor
                    } else {
                        &sprite.1.unwrap().anchor
                    };
                    // Hit box in sprite coordinate system

                    let extents = custom_size.or_else(|| {
                        if image.0.is_some() {
                            images.get(image.0.unwrap()).map(|f| f.size())
                        } else {
                            texture_atlas
                                .get(image.1.unwrap())
                                .map(|f| f.textures[sprite.1.unwrap().index].size())
                        }
                    })?;
                    let center = -anchor.as_vec() * extents;
                    let rect = Rect::from_center_half_size(center, extents / 2.0);

                    // Transform cursor pos to sprite coordinate system
                    let cursor_pos_sprite = sprite_transform
                        .affine()
                        .inverse()
                        .transform_point3((cursor_pos_world, 0.0).into());

                    let is_cursor_in_sprite = rect.contains(cursor_pos_sprite.truncate());
                    blocked = is_cursor_in_sprite
                        && sprite_focus.map(|p| p.should_block_lower) != Some(false);

                    is_cursor_in_sprite.then_some((
                        entity,
                        HitData::new(cam_entity, sprite_transform.translation().z, None, None),
                    ))
                },
            )
            .collect();

        let order = camera.order as f32;
        output.send(PointerHits::new(*pointer, picks, order))
    }
}
