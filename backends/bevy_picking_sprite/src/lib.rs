//! A raycasting backend for [`bevy_sprite`](bevy::sprite).

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_picking_core::backend::prelude::*;

/// Commonly used imports for the [`bevy_picking_ui`](crate) crate.
pub mod prelude {
    pub use crate::SpriteBackend;
}

/// Adds picking support for [`bevy_ui`](bevy::ui)
#[derive(Clone)]
pub struct SpriteBackend;
impl PickingBackend for SpriteBackend {}
impl Plugin for SpriteBackend {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .label(PickStage::Backend)
                .with_system(sprite_picking),
        );
    }
}

/// Computes the UI node entities under each pointer
pub fn sprite_picking(
    pointers: Query<(&PointerId, &PointerLocation)>,
    images: Res<Assets<Image>>,
    windows: Res<Windows>,
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
    for (pointer, location) in pointers.iter().filter_map(|(pointer, pointer_location)| {
        pointer_location.location().map(|loc| (pointer, loc))
    }) {
        let cursor_position = location.position;
        let mut blocked = false;

        let over_list = sprite_query
            .iter()
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
                        .or_else(|| images.get(image).and_then(|f| Some(f.size())))
                        .map(|size| size / 2.0)?;

                    let anchor_offset = sprite.anchor.as_vec() * extents;

                    let target = if let Some(t) =
                        location.target.get_render_target_info(&windows, &images)
                    {
                        t.physical_size.as_vec2() * t.scale_factor as f32
                    } else {
                        return None;
                    };

                    let min = sprite_position - extents + anchor_offset + target / 2.0;
                    let max = sprite_position + extents + anchor_offset + target / 2.0;

                    let contains_cursor = (min.x..max.x).contains(&cursor_position.x)
                        && (min.y..max.y).contains(&cursor_position.y);

                    contains_cursor.then_some(EntityDepth {
                        entity,
                        depth: position.z,
                    })
                },
            )
            .collect::<Vec<_>>();

        output.send(EntitiesUnderPointer {
            pointer: *pointer,
            over_list,
        })
    }
}
