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
    tasks::ComputeTaskPool,
    window::{CursorMoved, WindowId},
    utils::tracing::trace_span,
};
use core::convert::TryInto;
use std::collections::HashMap;

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PickState>()
            .init_resource::<PickHighlightParams>()
            .add_system(build_rays.system())
            .add_system(pick_mesh.system());
    }
}

pub struct PickState {
    /// Map of the single pick ray associated with each pick group
    ray_map: HashMap<Group, Ray3d>,
    ordered_pick_list_map: HashMap<Group, Vec<(Entity, Intersection)>>,
    pub enabled: bool,
    pub last_cursor_pos: Vec2,
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
            last_cursor_pos: Vec2::zero(),
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

/// Used to group pickable entities with pick rays
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Group(pub u8);

impl core::ops::Deref for Group {
    type Target = u8;
    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

impl Default for Group {
    fn default() -> Self {
        Group(0)
    }
}

/// Marks a Mesh entity as pickable
#[derive(Debug)]
pub struct PickableMesh {
    groups: Vec<Group>,
    intersections: HashMap<Group, Option<Intersection>>,
    bounding_sphere: Option<BoundingSphere>,
}

impl PickableMesh {
    /// Create a new PickableMesh with the specified pick group.
    pub fn new(groups: Vec<Group>) -> Self {
        let mut picks: HashMap<Group, Option<Intersection>> = HashMap::new();
        for group in &groups {
            picks.insert(*group, None);
        }
        PickableMesh {
            groups,
            intersections: picks,
            bounding_sphere: None,
        }
    }
    /// Returns the nearest intersection of the PickableMesh in the provided group.
    pub fn intersection(&self, group: &Group) -> Result<&Option<Intersection>, String> {
        self.intersections
            .get(group)
            .ok_or(format!("PickableMesh does not belong to group {}", **group))
    }
    pub fn with_bounding_sphere(&self, mesh: &Mesh) -> Self {
        PickableMesh {
            groups: self.groups.clone(),
            intersections: self.intersections.clone(),
            bounding_sphere: Some(BoundingSphere::from(mesh)),
        }
    }
}

impl Default for PickableMesh {
    fn default() -> Self {
        let mut picks = HashMap::new();
        picks.insert(Group::default(), None);
        PickableMesh {
            groups: [Group::default()].into(),
            bounding_sphere: None,
            intersections: picks,
        }
    }
}

/// Specifies the method used to generate pick rays
#[derive(Debug)]
pub enum PickMethod {
    /// Use cursor events to get coordinatess  relative to a camera
    CameraCursor(WindowId, UpdatePicks),
    /// Manually specify screen coordinatess relative to a camera
    CameraScreenSpace(Vec2),
    /// Use a tranform in world space to define pick ray
    Transform,
}

#[derive(Debug, Clone, Copy)]
pub enum UpdatePicks {
    Always,
    OnMouseEvent,
}

/// Marks an entity to be used for picking
pub struct PickSource {
    pub groups: Option<Vec<Group>>,
    pub pick_method: PickMethod,
    pub cursor_events: EventReader<CursorMoved>,
}

impl PickSource {
    pub fn new(group: Vec<Group>, pick_method: PickMethod) -> Self {
        PickSource {
            groups: Some(group),
            pick_method,
            ..Default::default()
        }
    }
    pub fn with_group(self, new_group: Group) -> Self {
        let new_groups = match self.groups {
            Some(group) => {
                let mut new_groups = group;
                new_groups.push(new_group);
                Some(new_groups)
            }
            None => Some(vec![new_group]),
        };
        PickSource {
            groups: new_groups,
            ..self
        }
    }
    pub fn with_pick_method(mut self, pick_method: PickMethod) -> Self {
        self.pick_method = pick_method;
        self
    }
}

impl Default for PickSource {
    fn default() -> Self {
        PickSource {
            groups: Some(vec![Group::default()]),
            pick_method: PickMethod::CameraCursor(WindowId::primary(), UpdatePicks::Always),
            cursor_events: EventReader::default(),
        }
    }
}

fn build_rays(
    // Resources
    mut pick_state: ResMut<PickState>,
    cursor: Res<Events<CursorMoved>>,
    windows: Res<Windows>,
    // Queries
    mut pick_source_query: Query<(&mut PickSource, &Transform, Option<&Camera>)>,
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
            // PickMethod::CameraCursor(window_id) => {
            PickMethod::CameraCursor(window_id, update_picks) => {
                // Option<Camera> allows us to query entities that may or may not have a camera. This pick method requres a camera!
                let projection_matrix = match camera {
                    Some(camera) => camera.projection_matrix,
                    None => panic!("The PickingSource in group(s) {:?} has a {:?} but no associated Camera component", group_numbers, pick_source.pick_method),
                };
                // Get the cursor position
                let cursor_pos_screen: Vec2 = match pick_source.cursor_events.latest(&cursor) {
                    Some(cursor_moved) => {
                        if cursor_moved.id == window_id {
                            cursor_moved.position
                        } else {
                            continue;
                        }
                    }
                    None => match update_picks {
                        UpdatePicks::Always => pick_state.last_cursor_pos,
                        UpdatePicks::OnMouseEvent => {
                            continue;
                        }
                    },
                };
                pick_state.last_cursor_pos = cursor_pos_screen;

                // Get current screen size
                let window = windows.get(window_id).unwrap();
                let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);

                // Normalized device coordinates (NDC) describes cursor position from (-1, -1, -1) to (1, 1, 1)
                let cursor_pos_ndc: Vec3 =
                    ((cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0])).extend(1.0);

                let camera_matrix = transform.compute_matrix();
                let (_, _, camera_position) = camera_matrix.to_scale_rotation_translation();

                let ndc_to_world: Mat4 = camera_matrix * projection_matrix.inverse();
                let cursor_position: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc);

                let ray_direction = cursor_position - camera_position;

                let pick_ray = Ray3d::new(camera_position, ray_direction);

                for group in group_numbers {
                    if pick_state.ray_map.insert(group, pick_ray).is_some() {
                        panic!(
                            "Multiple PickingSources have been added to pick group: {:?}",
                            group
                        );
                    }
                }
            }
            // Use the camera and specified screen cordinates to generate a ray
            PickMethod::CameraScreenSpace(coordinates_ndc) => {
                let projection_matrix = match camera {
                    Some(camera) => camera.projection_matrix,
                    None => panic!("The PickingSource in group(s) {:?} has a {:?} but no associated Camera component", group_numbers, pick_source.pick_method),
                };
                let cursor_pos_ndc: Vec3 = coordinates_ndc.extend(1.0);
                let camera_matrix = transform.compute_matrix();
                let (_, _, camera_position) = camera_matrix.to_scale_rotation_translation();

                let ndc_to_world: Mat4 = camera_matrix * projection_matrix.inverse();
                let cursor_position: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc);

                let ray_direction = cursor_position - camera_position;

                let pick_ray = Ray3d::new(camera_position, ray_direction);

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
                let source_transform = transform.compute_matrix();
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
    pool: Res<ComputeTaskPool>,
    // Queries
    mut mesh_query: Query<(
        &Handle<Mesh>,
        &GlobalTransform,
        &mut PickableMesh,
        Entity,
        &Visible,
    )>,
) {
    let sphere_check = trace_span!("sphere check");
    let raycast = trace_span!("raycast");
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
    for (mesh_handle, transform, mut pickable, entity, draw) in &mut mesh_query.iter_mut() {
        if !draw.is_visible {
            continue;
        }

        // Check for a pick ray in each pick group(s) this mesh belongs to
        let mut pick_rays: Vec<(Group, Ray3d)> = Vec::new();
        for group in &pickable.groups {
            if let Some(ray) = pick_state.ray_map.get(group) {
                pick_rays.push((*group, *ray));
            }
        }
        if pick_rays.is_empty() {
            continue;
        }

        // Cull pick rays that don't intersect the bounding sphere
        pick_rays.retain(|(_group, pick_ray)| {
            let _sphere_guard = sphere_check.enter();
            if let Some(sphere) = &pickable.bounding_sphere {
                let det = (pick_ray
                    .direction()
                    .dot(*pick_ray.origin() - (sphere.origin() + transform.translation)))
                .powi(2)
                    - (Vec3::length_squared(
                        *pick_ray.origin() - (sphere.origin() + transform.translation),
                    ) - sphere.radius().powi(2));
                det >= 0.0
            } else {
                true
            }
        });

        // Use the mesh handle to get a reference to a mesh asset
        if let Some(mesh) = meshes.get_mut(mesh_handle) {
            if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
                continue;
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
