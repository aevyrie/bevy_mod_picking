//! A raycasting backend for [`bevy_sprite`](bevy::sprite).

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use std::cmp::Ordering;

use bevy::ui::FocusPolicy;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_sprite`](crate) crate.
pub mod prelude {
    pub use crate::SpriteBackend;
}

/// Adds picking support for [`bevy_sprite`](bevy::sprite)
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
    sprite_query: Query<(
        Entity,
        &Sprite,
        &Handle<Image>,
        &GlobalTransform,
        &ComputedVisibility,
        Option<&FocusPolicy>,
    )>,
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
            .find(|(_, camera, _)| {
                camera
                    .target
                    .normalize(Some(primary_window.single()))
                    .unwrap()
                    == location.target
            }) else {
                continue;
            };

        let Some(cursor_pos_world) =
            camera.viewport_to_world_2d(cam_transform, location.position) else {
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
                    let position = sprite_transform.translation();
                    let half_extents = sprite
                        .custom_size
                        .or_else(|| images.get(image).map(|f| f.size()))
                        .map(|size| size / 2.0)?;
                    let center = position.truncate() + (sprite.anchor.as_vec() * half_extents);
                    let rect = Rect::from_center_half_size(center, half_extents);

                    let is_cursor_in_sprite = rect.contains(cursor_pos_world);
                    blocked = is_cursor_in_sprite && sprite_focus != Some(&FocusPolicy::Pass);

                    is_cursor_in_sprite.then_some((
                        entity,
                        HitData {
                            camera: cam_entity,
                            depth: position.z,
                            position: None,
                            normal: None,
                        },
                    ))
                },
            )
            .collect();

        output.send(PointerHits {
            pointer: *pointer,
            picks,
            order: 0,
        })
    }
}
