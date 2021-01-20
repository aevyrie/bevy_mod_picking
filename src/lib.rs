mod bounding;
mod debug;
mod highlight;
mod interactable;
mod raycast;
mod select;

pub use crate::{
    debug::DebugPickingPlugin,
    highlight::{HighlightablePickMesh, PickHighlightParams},
    interactable::{HoverEvents, InteractableMesh, InteractablePickingPlugin, MouseDownEvents},
    select::SelectablePickMesh,
};

use crate::bounding::*;
use crate::raycast::*;
use bevy::{
    prelude::*,
    render::{
        camera::Camera,
        mesh::{Indices, Mesh, VertexAttributeValues},
        pipeline::PrimitiveTopology,
    },
    window::CursorMoved,
};
use core::convert::TryInto;
use std::{collections::HashMap, marker::PhantomData};

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PickHighlightParams>()
            .add_system(build_bound_sphere.system())
            .add_stage_after(stage::POST_UPDATE, "picking", SystemStage::serial())
            .add_system_to_stage("picking", update_raycast.system());
    }
}

/// Holds computed intersection information
#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub struct Intersection {
    normal: Ray3d,
    pick_distance: f32,
    triangle: Triangle,
}
impl Intersection {
    fn new(normal: Ray3d, pick_distance: f32, triangle: Triangle) -> Self {
        Intersection {
            normal,
            pick_distance,
            triangle,
        }
    }
    /// Position vector describing the intersection position.
    pub fn position(&self) -> &Vec3 {
        self.normal.origin()
    }
    /// Unit vector describing the normal of the intersected triangle.
    pub fn normal(&self) -> &Vec3 {
        self.normal.direction()
    }
    /// Distance from the picking source to the entity.
    pub fn distance(&self) -> f32 {
        self.pick_distance
    }
    /// Triangle that was intersected with in World coordinates
    pub fn world_triangle(&self) -> Triangle {
        self.triangle
    }
}

/// Marks a Mesh entity as pickable
#[derive(Debug)]
pub struct PickableMesh<T> {
    phantom: PhantomData<T>,
}

/// Specifies the method used to generate pick rays
#[derive(Debug)]
pub enum PickMethod {
    /// Use cursor events to get coordinates  relative to a camera
    CameraCursor(UpdatePicks, EventReader<CursorMoved>),
    /// Manually specify screen coordinates relative to a camera
    CameraScreenSpace(Vec2),
    /// Use a tranform in world space to define pick ray
    Transform,
}

// TODO
// instead of making user specify when to update the picks, have it be event driven in the bevy ecs system
// basically, the user is responsible for triggering events. Need a way of having a default every frame method

#[derive(Debug, Clone, Copy)]
pub enum UpdatePicks {
    EveryFrame(Vec2),
    OnMouseEvent,
}

pub struct PickSource<T> {
    pub pick_method: PickMethod,
    ray: Option<Ray3d>,
    intersections: Option<Vec<(Entity, Intersection)>>,
    _phantom: PhantomData<T>,
}

impl<T> PickSource<T> {
    pub fn new(pick_method: PickMethod) -> Self {
        PickSource {
            pick_method,
            ray: None,
            ..Default::default()
        }
    }
    pub fn intersect_list(&self) -> Option<Vec<(Entity, Intersection)>> {
        self.intersections
    }
    pub fn intersect_top(&self) -> Option<(Entity, Intersection)> {
        self.intersections?.first()
    }
}

impl<T> Default for PickSource<T> {
    fn default() -> Self {
        PickSource {
            pick_method: PickMethod::CameraCursor(
                UpdatePicks::EveryFrame(Vec2::default()),
                EventReader::default(),
            ),
            ..Default::default()
        }
    }
}

fn update_raycast<T>(
    // Resources
    mut meshes: ResMut<Assets<Mesh>>,
    cursor: Res<Events<CursorMoved>>,
    windows: Res<Windows>,
    // Queries
    mut pick_source_query: Query<(
        &mut PickSource<T>,
        Option<&GlobalTransform>,
        Option<&Camera>,
    )>,
    mut mesh_query: Query<(
        &Handle<Mesh>,
        &GlobalTransform,
        &mut PickableMesh<T>,
        Entity,
        &Visible,
    )>,
) {
    if pick_source_query.iter().count() > 1 {
        panic!("Multiple PickSource components of the same type exist");
    }

    // Generate a ray for the picking source based on the pick method
    for (mut pick_source, transform, camera) in &mut pick_source_query.iter_mut() {
        pick_source.ray = match pick_source.pick_method {
            // Use cursor events and specified window/camera to generate a ray
            PickMethod::CameraCursor(update_picks, event_reader) => {
                let camera = match camera {
                    Some(camera) => camera,
                    None => panic!(
                        "The PickingSource has a {:?} but no associated Camera component",
                        pick_source.pick_method
                    ),
                };
                let cursor_latest = match event_reader.latest(&cursor) {
                    Some(cursor_moved) => {
                        if cursor_moved.id == camera.window {
                            Some(cursor_moved)
                        } else {
                            None
                        }
                    }
                    None => None,
                };
                let cursor_pos_screen: Vec2 = match update_picks {
                    UpdatePicks::EveryFrame(cached_pos) => match cursor_latest {
                        Some(cursor_moved) => {
                            //Updated the cached cursor position
                            pick_source.pick_method = PickMethod::CameraCursor(
                                camera.window,
                                UpdatePicks::EveryFrame(cursor_moved.position),
                            );
                            cursor_moved.position
                        }
                        None => cached_pos,
                    },
                    UpdatePicks::OnMouseEvent => match cursor_latest {
                        Some(cursor_moved) => cursor_moved.position,
                        None => continue,
                    },
                };

                // Get current screen size
                let window = windows
                    .get(camera.window)
                    .unwrap_or_else(|| panic!("WindowId {} does not exist", camera.window));
                let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);

                // Normalized device coordinates (NDC) describes cursor position from (-1, -1, -1) to (1, 1, 1)
                let cursor_ndc = (cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
                let cursor_pos_ndc_near: Vec3 = cursor_ndc.extend(-1.0);
                let cursor_pos_ndc_far: Vec3 = cursor_ndc.extend(1.0);

                let camera_matrix = match transform {
                    Some(matrix) => matrix,
                    None => panic!(
                        "The PickingSource {:?} has no associated GlobalTransform component",
                        pick_source
                    ),
                }
                .compute_matrix();

                let ndc_to_world: Mat4 = camera_matrix * camera.projection_matrix.inverse();
                let cursor_pos_near: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc_near);
                let cursor_pos_far: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc_far);

                let ray_direction = cursor_pos_far - cursor_pos_near;

                Some(Ray3d::new(cursor_pos_near, ray_direction))
            }
            // Use the camera and specified screen coordinates to generate a ray
            PickMethod::CameraScreenSpace(coordinates_ndc) => {
                let projection_matrix = match camera {
                    Some(camera) => camera.projection_matrix,
                    None => panic!(
                        "The PickingSource has a {:?} but no associated Camera component",
                        pick_source.pick_method
                    ),
                };
                let cursor_pos_ndc_near: Vec3 = coordinates_ndc.extend(-1.0);
                let cursor_pos_ndc_far: Vec3 = coordinates_ndc.extend(1.0);
                let camera_matrix = match transform {
                    Some(matrix) => matrix,
                    None => panic!(
                        "The PickingSource has a {:?} but no associated GlobalTransform component",
                        pick_source.pick_method
                    ),
                }
                .compute_matrix();

                let ndc_to_world: Mat4 = camera_matrix * projection_matrix.inverse();
                let cursor_pos_near: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc_near);
                let cursor_pos_far: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc_far);

                let ray_direction = cursor_pos_far - cursor_pos_near;

                Some(Ray3d::new(cursor_pos_near, ray_direction))
            }
            // Use the specified transform as the origin and direction of the ray
            PickMethod::Transform => {
                let pick_position_ndc = Vec3::from([0.0, 0.0, 1.0]);
                let source_transform = match transform {
                    Some(matrix) => matrix,
                    None => panic!(
                        "The PickingSource has a {:?} but no associated GlobalTransform component",
                        pick_source.pick_method
                    ),
                }
                .compute_matrix();
                let pick_position = source_transform.transform_point3(pick_position_ndc);

                let (_, _, source_origin) = source_transform.to_scale_rotation_translation();
                let ray_direction = pick_position - source_origin;

                Some(Ray3d::new(source_origin, ray_direction))
            }
        };

        if let Some(ray) = pick_source.ray {
            // Create spans for tracing
            let ray_cull = info_span!("ray culling");
            let raycast = info_span!("raycast");

            // Iterate through each pickable mesh in the scene
            //mesh_query.par_iter_mut(32).for_each(&pool,|(mesh_handle, transform, mut pickable, entity, draw)| {},);
            for (mesh_handle, transform, mut pickable, entity, visibility) in
                &mut mesh_query.iter_mut()
            {
                if !visibility.is_visible {
                    continue;
                }

                // Check for a pick ray in each pick group(s) this mesh belongs to
                let _ray_cull_guard = ray_cull.enter();
                // Cull pick rays that don't intersect the bounding sphere
                // NOTE: this might cause stutters on load because bound spheres won't be loaded
                // and picking will be brute forcing.
                if let BoundVol::Loaded(sphere) = &pickable.bounding_sphere {
                    let scaled_radius = 1.01 * sphere.radius() * transform.scale.max_element();
                    let translated_origin =
                        sphere.origin() * transform.scale + transform.translation;
                    let det = (ray.direction().dot(*ray.origin() - translated_origin)).powi(2)
                        - (Vec3::length_squared(*ray.origin() - translated_origin)
                            - scaled_radius.powi(2));
                    if det < 0.0 {
                        continue; // Ray does not intersect the bounding sphere - skip entity
                    }
                }

                // Use the mesh handle to get a reference to a mesh asset
                if let Some(mesh) = meshes.get_mut(mesh_handle) {
                    if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
                        panic!("bevy_mod_picking only supports TriangleList topology");
                    }

                    let _raycast_guard = raycast.enter();
                    // Get the vertex positions from the mesh reference resolved from the mesh handle
                    let vertex_positions: Vec<[f32; 3]> =
                        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                            None => panic!("Mesh does not contain vertex positions"),
                            Some(vertex_values) => match &vertex_values {
                                VertexAttributeValues::Float3(positions) => positions.clone(),
                                _ => panic!("Unexpected vertex types in ATTRIBUTE_POSITION"),
                            },
                        };

                    if let Some(indices) = &mesh.indices() {
                        // Iterate over the list of pick rays that belong to the same group as this mesh
                        let mesh_to_world = transform.compute_matrix();
                        let new_intersection = match indices {
                            Indices::U16(vector) => ray_mesh_intersection(
                                &mesh_to_world,
                                &vertex_positions,
                                &ray,
                                vector,
                            ),
                            Indices::U32(vector) => ray_mesh_intersection(
                                &mesh_to_world,
                                &vertex_positions,
                                &ray,
                                vector,
                            ),
                        };
                        pick_source.intersections.push((entity, new_intersection));
                    } else {
                        // If we get here the mesh doesn't have an index list!
                        panic!(
                            "No index matrix found in mesh {:?}\n{:?}",
                            mesh_handle, mesh
                        );
                    }
                }
            }

            // Sort the pick list
            if let Some(intersection_list) = pick_source.intersections {
                intersection_list.sort_by(|a, b| {
                    a.1.pick_distance
                        .partial_cmp(&b.1.pick_distance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
    }
}

fn ray_mesh_intersection<T: TryInto<usize> + Copy>(
    mesh_to_world: &Mat4,
    vertex_positions: &[[f32; 3]],
    pick_ray: &Ray3d,
    indices: &[T],
) -> Option<Intersection> {
    // The ray cast can hit the same mesh many times, so we need to track which hit is
    // closest to the camera, and record that.
    let mut min_pick_distance = f32::MAX;
    let mut pick_intersection: Option<Intersection> = None;

    // Make sure this chunk has 3 vertices to avoid a panic.
    if indices.len() % 3 == 0 {
        // Now that we're in the vector of vertex indices, we want to look at the vertex
        // positions for each triangle, so we'll take indices in chunks of three, where each
        // chunk of three indices are references to the three vertices of a triangle.
        for index in indices.chunks(3) {
            // Construct a triangle in world space using the mesh data
            let mut world_vertices: [Vec3; 3] = [Vec3::zero(), Vec3::zero(), Vec3::zero()];
            for i in 0..3 {
                if let Ok(vertex_index) = index[i].try_into() {
                    world_vertices[i] =
                        mesh_to_world.transform_point3(Vec3::from(vertex_positions[vertex_index]));
                } else {
                    panic!("Failed to convert index into usize.");
                }
            }
            let world_triangle = Triangle::from(world_vertices);
            // Run the raycast on the ray and triangle
            if let Some(intersection) =
                ray_triangle_intersection(pick_ray, &world_triangle, RaycastAlgorithm::default())
            {
                let distance: f32 = (*intersection.origin() - *pick_ray.origin()).length().abs();
                if distance < min_pick_distance {
                    min_pick_distance = distance;
                    pick_intersection =
                        Some(Intersection::new(intersection, distance, world_triangle));
                }
            }
        }
    }
    pick_intersection
}
