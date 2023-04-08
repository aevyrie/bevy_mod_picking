//! A raycasting backend for [`bevy_sprite`](bevy::sprite).

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use std::cmp::Ordering;

use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_picking_core::backend::{prelude::*};

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
    windows: Query<(Entity, &Window)>,
    images: Res<Assets<Image>>,
    sprite_query: Query<
        (
            Entity,
            &Sprite,
            &Handle<Image>,
            &GlobalTransform,
            &ComputedVisibility,
            Option<&FocusPolicy>,
        ),
        With<Pickable>,
    >,
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
        let cursor_position = location.position;
        let mut blocked = false;

        let over_list = sorted_sprites
            .iter()
            .copied()
            .filter_map(
                |(entity, sprite, image, global_transform, visibility, focus)| {
                    if blocked || !visibility.is_visible() {
                        return None;
                    }

                    blocked = focus != Some(&FocusPolicy::Pass);

                    let position = global_transform.translation();
                    let sprite_position = position.truncate();

                    let extents = sprite
                        .custom_size
                        .or_else(|| images.get(image).map(|f| f.size()))
                        .map(|size| size / 2.0)?;

                    let anchor_offset = sprite.anchor.as_vec() * extents;

                    let target = if let Some(t) =
                        location.target.get_render_target_info(&windows, &images)
                    {
                        t.physical_size.as_vec2() / t.scale_factor as f32
                    } else {
                        return None;
                    };

                    let min = sprite_position - extents + anchor_offset + target / 2.0;
                    let max = sprite_position + extents + anchor_offset + target / 2.0;

                    let contains_cursor = (min.x..max.x).contains(&cursor_position.x)
                        && (min.y..max.y).contains(&cursor_position.y);

                    contains_cursor.then_some((
                        entity,
                        PickData {
                            depth: position.z,
                            normal: None,
                        },
                    ))
                },
            )
            .collect::<Vec<_>>();

        output.send(EntitiesUnderPointer {
            pointer: *pointer,
            picks: over_list,
        })
    }
}
