//! A raycasting backend for `bevy_mod_picking` that uses `rapier` for raycasting.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Adds the `rapier` raycasting picking backend to your app.
pub struct RapierPlugin;
impl Plugin for RapierPlugin {
    fn build(&self, _app: &mut App) {}
}

fn cast_ray(
    mut commands: Commands,
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    // We will color in read the colliders hovered by the mouse.
    for (camera, camera_transform) in cameras.iter() {
        // First, compute a ray from the mouse position.
        let (ray_pos, ray_dir) =
            ray_from_mouse_position(windows.get_primary().unwrap(), camera, camera_transform);

        // Then cast the ray.
        let hit = rapier_context.cast_ray(
            ray_pos,
            ray_dir,
            f32::MAX,
            true,
            QueryFilter::only_dynamic(),
        );

        if let Some((entity, _toi)) = hit {
            // Color in blue the entity we just hit.
            // Because of the query filter, only colliders attached to a dynamic body
            // will get an event.
            let color = Color::BLUE;
            commands.entity(entity).insert(ColliderDebugColor(color));
        }
    }
}

// Credit to @doomy on discord.
fn ray_from_mouse_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> (Vec3, Vec3) {
    let mouse_position = window.cursor_position().unwrap_or(Vec2::new(0.0, 0.0));

    let x = 2.0 * (mouse_position.x / window.width() as f32) - 1.0;
    let y = 2.0 * (mouse_position.y / window.height() as f32) - 1.0;

    let camera_inverse_matrix =
        camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let near = camera_inverse_matrix * Vec3::new(x, y, -1.0).extend(1.0);
    let far = camera_inverse_matrix * Vec3::new(x, y, 1.0).extend(1.0);

    let near = near.truncate() / near.w;
    let far = far.truncate() / far.w;
    let dir: Vec3 = far - near;
    (near, dir)
}
