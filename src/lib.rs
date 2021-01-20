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
    window::{CursorMoved, WindowId},
};
use core::convert::TryInto;
use std::{collections::HashMap, marker::PhantomData};

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PickState>()
            .init_resource::<PickHighlightParams>()
            .add_system(build_bound_sphere.system())
            .add_stage_after(stage::POST_UPDATE, "picking", SystemStage::serial())
            .add_system_to_stage("picking", build_rays.system())
            .add_system_to_stage("picking", pick_mesh.system());
    }
}

pub struct PickState {
    /// Map of the single pick ray associated with each pick group
    ray_map: Vec<Ray3d>,
    ordered_pick_list_map: HashMap<Group, Vec<(Entity, Intersection)>>,
    pub enabled: bool,
}

impl PickState {
    pub fn list(&self, group: Group) -> Option<&Vec<(Entity, Intersection)>> {
        self.ordered_pick_list_map.get(&group)
    }
    pub fn top(&self, group: Group) -> Option<&(Entity, Intersection)> {
        match self.ordered_pick_list_map.get(&group) {
            Some(list) => list.first(),
            None => None,
        }
    }
    pub fn top_all(&self) -> Option<Vec<(&Group, &Entity, &Intersection)>> {
        let mut result = Vec::new();
        for (group, picklist) in self.ordered_pick_list_map.iter() {
            if let Some(pick) = picklist.first() {
                let (entity, intersection) = pick;
                result.push((group, entity, intersection));
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
    fn empty_pick_list(&mut self) {
        for (_group, intersection_list) in self.ordered_pick_list_map.iter_mut() {
            intersection_list.clear();
        }
    }
}

impl Default for PickState {
    fn default() -> Self {
        PickState {
            ray_map: HashMap::new(),
            ordered_pick_list_map: HashMap::new(),
            enabled: true,
        }
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
    intersection: Option<Intersection>,
}

impl<T> PickableMesh<T> {    
    pub fn intersection(&self) -> Option<Intersection> {
        self.intersection
    }
}

impl<T> Default for PickableMesh<T> {
    fn default() -> Self {
        PickableMesh {
            intersection: None,
        }
    }
}

/// Specifies the method used to generate pick rays
#[derive(Debug)]
pub enum PickMethod {
    /// Use cursor events to get coordinates  relative to a camera
    CameraCursor(UpdatePicks, EventReader<CursorMoved>),
    /// Manually specify screen coordinates relative to a camera
    CameraScreenSpace(Vec2),
    /// Use a tranform in world space to define pick ray
    Transform(GlobalTransform),
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

fn build_rays(
    // Resources
    mut pick_state: ResMut<PickState>,
    cursor: Res<Events<CursorMoved>>,
    windows: Res<Windows>,
    // Queries
    mut pick_source_query: Query<(&mut PickSource, Option<&GlobalTransform>, Option<&Camera>)>,
) {
    // Collect and calculate pick_ray from all cameras
    pick_state.ray_map.clear();

    if !pick_state.enabled {
        return;
    }

    // Generate a ray for each picking source based on the pick method
    for (mut pick_source, transform, camera) in &mut pick_source_query.iter_mut() {
        let group_numbers = match &pick_source.groups {
            Some(groups) => groups.clone(),
            None => continue,
        };

        match pick_source.pick_method {
            // Use cursor events and specified window/camera to generate a ray
            PickMethod::CameraCursor(window_id, update_picks) => {
                // Option<Camera> allows us to query entities that may or may not have a camera. This pick method requires a camera!
                let projection_matrix = match camera {
                    Some(camera) => camera.projection_matrix,
                    None => panic!("The PickingSource in group(s) {:?} has a {:?} but no associated Camera component", group_numbers, pick_source.pick_method),
                };
                // Get the cursor position
                let cursor_latest = match pick_source.cursor_events.latest(&cursor) {
                    Some(cursor_moved) => {
                        if cursor_moved.id == window_id {
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
                                window_id,
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
                    .get(window_id)
                    .unwrap_or_else(|| panic!("WindowId {} does not exist", window_id));
                let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);

                // Normalized device coordinates (NDC) describes cursor position from (-1, -1, -1) to (1, 1, 1)
                let cursor_ndc = (cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
                let cursor_pos_ndc_near: Vec3 = cursor_ndc.extend(-1.0);
                let cursor_pos_ndc_far: Vec3 = cursor_ndc.extend(1.0);

                let camera_matrix = match transform {
                    Some(matrix) => matrix,
                    None => panic!("The PickingSource in group(s) {:?} has a {:?} but no associated GlobalTransform component", group_numbers, pick_source.pick_method),
                }.compute_matrix();

                let ndc_to_world: Mat4 = camera_matrix * projection_matrix.inverse();
                let cursor_pos_near: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc_near);
                let cursor_pos_far: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc_far);

                let ray_direction = cursor_pos_far - cursor_pos_near;

                let pick_ray = Ray3d::new(cursor_pos_near, ray_direction);

                for group in group_numbers {
                    if pick_state.ray_map.insert(group, pick_ray).is_some() {
                        panic!(
                            "Multiple PickingSources have been added to pick group: {:?}",
                            group
                        );
                    }
                }
            }
            // Use the camera and specified screen coordinates to generate a ray
            PickMethod::CameraScreenSpace(coordinates_ndc) => {
                let projection_matrix = match camera {
                    Some(camera) => camera.projection_matrix,
                    None => panic!("The PickingSource in group(s) {:?} has a {:?} but no associated Camera component", group_numbers, pick_source.pick_method),
                };
                let cursor_pos_ndc_near: Vec3 = coordinates_ndc.extend(-1.0);
                let cursor_pos_ndc_far: Vec3 = coordinates_ndc.extend(1.0);
                let camera_matrix = match transform {
                    Some(matrix) => matrix,
                    None => panic!("The PickingSource in group(s) {:?} has a {:?} but no associated GlobalTransform component", group_numbers, pick_source.pick_method),
                }.compute_matrix();

                let ndc_to_world: Mat4 = camera_matrix * projection_matrix.inverse();
                let cursor_pos_near: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc_near);
                let cursor_pos_far: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc_far);

                let ray_direction = cursor_pos_far - cursor_pos_near;

                let pick_ray = Ray3d::new(cursor_pos_near, ray_direction);

                for group in group_numbers {
                    if pick_state.ray_map.insert(group, pick_ray).is_some() {
                        panic!(
                            "Multiple PickingSources have been added to pick group: {:?}",
                            group
                        );
                    }
                }
            }
            // Use the specified transform as the origin and direction of the ray
            PickMethod::Transform => {
                let pick_position_ndc = Vec3::from([0.0, 0.0, 1.0]);
                let source_transform = match transform {
                    Some(matrix) => matrix,
                    None => panic!("The PickingSource in group(s) {:?} has a {:?} but no associated GlobalTransform component", group_numbers, pick_source.pick_method),
                }.compute_matrix();
                let pick_position = source_transform.transform_point3(pick_position_ndc);

                let (_, _, source_origin) = source_transform.to_scale_rotation_translation();
                let ray_direction = pick_position - source_origin;

                let pick_ray = Ray3d::new(source_origin, ray_direction);

                for group in group_numbers {
                    if pick_state.ray_map.insert(group, pick_ray).is_some() {
                        panic!(
                            "Multiple PickingSources have been added to pick group: {:?}",
                            group
                        );
                    }
                }
            }
        }
    }
}

fn pick_mesh(
    // Resources
    mut pick_state: ResMut<PickState>,
    mut meshes: ResMut<Assets<Mesh>>,
    //_pool: Res<ComputeTaskPool>,
    // Queries
    mut mesh_query: Query<(
        &Handle<Mesh>,
        &GlobalTransform,
        &mut PickableMesh,
        Entity,
        &Visible,
    )>,
) {
    // Create spans for tracing
    let ray_cull = info_span!("ray culling");
    let raycast = info_span!("raycast");

    // If picking is disabled, do not continue
    if !pick_state.enabled {
        pick_state.empty_pick_list();
        return;
    }

    // If there are no rays, then there is nothing to do here
    if pick_state.ray_map.is_empty() {
        return;
    } else {
        // Clear picks in list only if there are new picking rays, otherwise keep state same
        pick_state.empty_pick_list();
    }

    // Iterate through each pickable mesh in the scene
    //mesh_query.par_iter_mut(32).for_each(&pool,|(mesh_handle, transform, mut pickable, entity, draw)| {},);
    for (mesh_handle, transform, mut pickable, entity, visibility) in &mut mesh_query.iter_mut() {
        if !visibility.is_visible {
            continue;
        }

        // Check for a pick ray in each pick group(s) this mesh belongs to
        let pick_rays: Vec<(Group, Ray3d)> = {
            let _ray_cull_guard = ray_cull.enter();
            pickable
                .groups
                .iter()
                .filter_map(|group| {
                    if let Some(ray) = pick_state.ray_map.get(group) {
                        // Cull pick rays that don't intersect the bounding sphere
                        // NOTE: this might cause stutters on load because bound spheres won't be loaded
                        // and picking will be brute forcing.
                        if let BoundVol::Loaded(sphere) = &pickable.bounding_sphere {
                            let scaled_radius =
                                1.01 * sphere.radius() * transform.scale.max_element();
                            let translated_origin =
                                sphere.origin() * transform.scale + transform.translation;
                            let det = (ray.direction().dot(*ray.origin() - translated_origin))
                                .powi(2)
                                - (Vec3::length_squared(*ray.origin() - translated_origin)
                                    - scaled_radius.powi(2));
                            if det >= 0.0 {
                                Some((*group, *ray)) // Ray intersects the bounding sphere
                            } else {
                                None // Ray does not intersect the bounding sphere - discard
                            }
                        } else {
                            Some((*group, *ray)) // No bounding sphere present - can't discard
                        }
                    } else {
                        None // No ray present in the map for this group
                    }
                })
                .collect()
        };
        if pick_rays.is_empty() {
            continue;
        }

        // Use the mesh handle to get a reference to a mesh asset
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
                panic!("bevy_mod_picking only supports TriangleList topology");
            }

            let _raycast_guard = raycast.enter();
            // Get the vertex positions from the mesh reference resolved from the mesh handle
            let vertex_positions: Vec<[f32; 3]> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                None => panic!("Mesh does not contain vertex positions"),
                Some(vertex_values) => match &vertex_values {
                    VertexAttributeValues::Float3(positions) => positions.clone(),
                    _ => panic!("Unexpected vertex types in ATTRIBUTE_POSITION"),
                },
            };

            if let Some(indices) = &mesh.indices() {
                // Iterate over the list of pick rays that belong to the same group as this mesh
                for (pick_group, pick_ray) in pick_rays {
                    let mesh_to_world = transform.compute_matrix();
                    let new_intersection = match indices {
                        Indices::U16(vector) => ray_mesh_intersection(
                            &mesh_to_world,
                            &vertex_positions,
                            &pick_ray,
                            vector,
                        ),
                        Indices::U32(vector) => ray_mesh_intersection(
                            &mesh_to_world,
                            &vertex_positions,
                            &pick_ray,
                            vector,
                        ),
                    };

                    // Finished going through the current mesh, update pick states
                    if let Some(intersection) = new_intersection {
                        // Make sure the pick list map contains the key
                        match pick_state.ordered_pick_list_map.get_mut(&pick_group) {
                            Some(list) => list.push((entity, intersection)),
                            None => {
                                pick_state
                                    .ordered_pick_list_map
                                    .insert(pick_group, Vec::from([(entity, intersection)]));
                            }
                        }
                    }

                    pickable.intersections.insert(pick_group, new_intersection);
                }
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
    for (_group, intersection_list) in pick_state.ordered_pick_list_map.iter_mut() {
        intersection_list.sort_by(|a, b| {
            a.1.pick_distance
                .partial_cmp(&b.1.pick_distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
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
