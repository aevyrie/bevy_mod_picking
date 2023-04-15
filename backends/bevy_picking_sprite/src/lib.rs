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
impl PickingBackend for SpriteBackend {}
impl Plugin for SpriteBackend {
    fn build(&self, app: &mut App) {
        app.add_system(sprite_picking.in_set(PickSet::Backend));
    }
}

/// Checks if any sprite entities are under each pointer
pub fn sprite_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(Entity, &Camera)>,
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
    mut output: EventWriter<EntitiesUnderPointer>,
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
        let (cam_entity, camera) = cameras
            .iter()
            .find(|(_entity, camera)| {
                camera
                    .target
                    .normalize(Some(primary_window.single()))
                    .unwrap()
                    == location.target
            })
            .unwrap_or_else(|| panic!("No camera found associated with pointer {:?}.", pointer));
        let target_half_extents = if let Some(target) = camera.logical_target_size() {
            target / 2.0
        } else {
            continue;
        };
        let cursor_position = location.position;
        let cursor_pos_centered = cursor_position - target_half_extents;

        let picks: Vec<(Entity, PickData)> = sorted_sprites
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

                    let is_cursor_in_sprite = rect.contains(cursor_pos_centered);
                    blocked = is_cursor_in_sprite && sprite_focus != Some(&FocusPolicy::Pass);

                    is_cursor_in_sprite.then_some((
                        entity,
                        PickData {
                            camera: cam_entity,
                            depth: position.z,
                            position: None,
                            normal: None,
                        },
                    ))
                },
            )
            .collect();

        output.send(EntitiesUnderPointer {
            pointer: *pointer,
            picks,
            order: 0,
        })
    }
}
