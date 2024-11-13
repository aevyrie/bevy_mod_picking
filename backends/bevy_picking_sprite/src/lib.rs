//! A raycasting backend for [`bevy_sprite`].

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use std::cmp::Ordering;

use bevy_app::prelude::*;
use bevy_asset::prelude::*;
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;
use bevy_reflect::prelude::*;
use bevy_render::prelude::*;
use bevy_sprite::{Sprite, TextureAtlas, TextureAtlasLayout};
use bevy_transform::prelude::*;
use bevy_window::PrimaryWindow;

use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_sprite`](crate) crate.
pub mod prelude {
    pub use crate::SpriteBackend;
}

/// Runtime settings for the [`SpriteBackend`].
#[derive(Resource, Reflect)]
#[reflect(Resource, Default)]
pub struct SpriteBackendSettings {
    /// When set to `true` picking will ignore any part of a sprite which is transparent
    /// Off by default for backwards compatibility. This setting is provided to give you fine-grained
    /// control over if transparncy on sprites is ignored.
    pub passthrough_transparency: bool,
    /// How Opaque does part of a sprite need to be in order count as none-transparent (defaults to 245)
    pub transparency_cutoff: u8,
}

impl Default for SpriteBackendSettings {
    fn default() -> Self {
        Self {
            passthrough_transparency: true,
            transparency_cutoff: 10,
        }
    }
}

/// Adds picking support for [`bevy_sprite`].
#[derive(Clone)]
pub struct SpriteBackend;

impl Plugin for SpriteBackend {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteBackendSettings>()
            .add_systems(PreUpdate, sprite_picking.in_set(PickSet::Backend));
    }
}

/// Checks if any sprite entities are under each pointer
pub fn sprite_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera, &GlobalTransform, &OrthographicProjection)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    texture_atlas_layout: Res<Assets<TextureAtlasLayout>>,
    settings: Res<SpriteBackendSettings>,
    sprite_query: Query<
        (
            Entity,
            Option<&Sprite>,
            Option<&TextureAtlas>,
            Option<&Handle<Image>>,
            &GlobalTransform,
            Option<&Pickable>,
            &ViewVisibility,
        ),
        Or<(With<Sprite>, With<TextureAtlas>)>,
    >,
    mut output: EventWriter<PointerHits>,
) {
    let mut sorted_sprites: Vec<_> = sprite_query.iter().collect();
    sorted_sprites.sort_by(|a, b| {
        (b.4.translation().z)
            .partial_cmp(&a.4.translation().z)
            .unwrap_or(Ordering::Equal)
    });

    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let mut blocked = false;
        let Some((cam_entity, camera, cam_transform, cam_ortho)) = cameras
            .iter()
            .filter(|(_, camera, _, _)| camera.is_active)
            .find(|(_, camera, _, _)| {
                camera
                    .target
                    .normalize(Some(match primary_window.get_single() {
                        Ok(w) => w,
                        Err(_) => return false,
                    }))
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
            .filter(|(.., visibility)| visibility.get())
            .filter_map(
                |(entity, sprite, atlas, image, sprite_transform, pickable, ..)| {
                    if blocked {
                        return None;
                    }

                    // Hit box in sprite coordinate system
                    let (extents, anchor) = if let Some((sprite, atlas)) = sprite.zip(atlas) {
                        let extents = sprite.custom_size.or_else(|| {
                            texture_atlas_layout
                                .get(&atlas.layout)
                                .map(|f| f.textures[atlas.index].size().as_vec2())
                        })?;
                        let anchor = sprite.anchor.as_vec();
                        (extents, anchor)
                    } else if let Some((sprite, image)) = sprite.zip(image) {
                        let extents = sprite
                            .custom_size
                            .or_else(|| images.get(image).map(|f| f.size().as_vec2()))?;
                        let anchor = sprite.anchor.as_vec();
                        (extents, anchor)
                    } else {
                        return None;
                    };

                    let center = -anchor * extents;
                    let rect = Rect::from_center_half_size(center, extents / 2.0);

                    // Transform cursor pos to sprite coordinate system
                    let cursor_pos_sprite = sprite_transform
                        .affine()
                        .inverse()
                        .transform_point3((cursor_pos_world, 0.0).into());

                    let is_cursor_in_sprite = rect.contains(cursor_pos_sprite.truncate());

                    let cursor_in_valid_pixels_of_sprite = is_cursor_in_sprite
                        && settings.passthrough_transparency
                        && (image.is_none() || {
                            let texture: &Image = image.and_then(|i| images.get(i))?;
                            // If using a texture atlas, grab the offset of the current sprite index. (0,0) otherwise
                            let texture_rect = atlas
                                .and_then(|atlas| {
                                    texture_atlas_layout
                                        .get(&atlas.layout)
                                        .map(|f| f.textures[atlas.index])
                                })
                                .or(Some(URect::new(0, 0, texture.width(), texture.height())))?;
                            let texture_position =
                                texture_rect.center() + cursor_pos_sprite.truncate().as_uvec2();
                            let pixel_index = (texture_position.y * texture.width()
                                + texture_position.x)
                                as usize;
                            if let Some(pixel_data) =
                                texture.data.get(pixel_index * 4..(pixel_index * 4 + 4))
                            {
                                let transparency = pixel_data[3];
                                println!("pixel transparency: {}", transparency);
                                transparency > settings.transparency_cutoff
                            } else {
                                false
                            }
                        });

                    blocked = cursor_in_valid_pixels_of_sprite
                        && pickable.map(|p| p.should_block_lower) != Some(false);

                    // HitData requires a depth as calculated from the camera's near clipping plane
                    let depth = -cam_ortho.near - sprite_transform.translation().z;

                    cursor_in_valid_pixels_of_sprite
                        .then_some((entity, HitData::new(cam_entity, depth, None, None)))
                },
            )
            .collect();

        let order = camera.order as f32;
        output.send(PointerHits::new(*pointer, picks, order));
    }
}
