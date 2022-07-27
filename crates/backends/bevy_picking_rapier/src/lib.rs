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

// fn cast_ray(
//     mut commands: Commands,
//     windows: Res<Windows>,
//     rapier_context: Res<RapierContext>,
//     cameras: Query<(&Camera, &GlobalTransform)>,
// ) {
//     // We will color in read the colliders hovered by the mouse.
//     for (camera, camera_transform) in cameras.iter() {
//         // First, compute a ray from the mouse position.
//         let (ray_pos, ray_dir) =
//             ray_from_mouse_position(windows.get_primary().unwrap(), camera, camera_transform);

//         // Then cast the ray.
//         let hit = rapier_context.cast_ray(
//             ray_pos,
//             ray_dir,
//             f32::MAX,
//             true,
//             QueryFilter::only_dynamic(),
//         );

//         if let Some((entity, _toi)) = hit {
//             // Color in blue the entity we just hit.
//             // Because of the query filter, only colliders attached to a dynamic body
//             // will get an event.
//             let color = Color::BLUE;
//             commands.entity(entity).insert(ColliderDebugColor(color));
//         }
//     }
// }

// // Credit to @doomy on discord.
// fn ray_from_mouse_position(
//     window: &Window,
//     camera: &Camera,
//     camera_transform: &GlobalTransform,
// ) -> (Vec3, Vec3) {
//     let mouse_position = window.cursor_position().unwrap_or(Vec2::new(0.0, 0.0));

//     let x = 2.0 * (mouse_position.x / window.width() as f32) - 1.0;
//     let y = 2.0 * (mouse_position.y / window.height() as f32) - 1.0;

//     let camera_inverse_matrix =
//         camera_transform.compute_matrix() * camera.projection_matrix().inverse();
//     let near = camera_inverse_matrix * Vec3::new(x, y, -1.0).extend(1.0);
//     let far = camera_inverse_matrix * Vec3::new(x, y, 1.0).extend(1.0);

//     let near = near.truncate() / near.w;
//     let far = far.truncate() / far.w;
//     let dir: Vec3 = far - near;
//     (near, dir)
// }

// pub fn from_screenspace(
//     cursor_pos_screen: Vec2,
//     camera: &Camera,
//     camera_transform: &GlobalTransform,
// ) -> Option<Self> {
//     let view = camera_transform.compute_matrix();
//     let screen_size = match camera.logical_target_size() {
//         Some(s) => s,
//         None => {
//             error!(
//                 "Unable to get screen size for RenderTarget {:?}",
//                 camera.target
//             );
//             return None;
//         }
//     };
//     let projection = camera.projection_matrix();

//     // 2D Normalized device coordinate cursor position from (-1, -1) to (1, 1)
//     let cursor_ndc = (cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
//     let ndc_to_world: Mat4 = view * projection.inverse();
//     let world_to_ndc = projection * view;
//     let is_orthographic = projection.w_axis[3] == 1.0;

//     // Calculate the camera's near plane using the projection matrix
//     let projection = projection.to_cols_array_2d();
//     let camera_near = (2.0 * projection[3][2]) / (2.0 * projection[2][2] - 2.0);

//     // Compute the cursor position at the near plane. The bevy camera looks at -Z.
//     let ndc_near = world_to_ndc.transform_point3(-Vec3::Z * camera_near).z;
//     let cursor_pos_near = ndc_to_world.transform_point3(cursor_ndc.extend(ndc_near));

//     // Compute the ray's direction depending on the projection used.
//     let ray_direction = match is_orthographic {
//         true => view.transform_vector3(-Vec3::Z), // All screenspace rays are parallel in ortho
//         false => cursor_pos_near - camera_transform.translation(), // Direction from camera to cursor
//     };

//     Some(Ray3d::new(cursor_pos_near, ray_direction))
// }
