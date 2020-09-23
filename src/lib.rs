mod debug;
mod highlight;
mod raycast;
mod select;

pub use crate::{
    debug::DebugPickingPlugin,
    highlight::{HighlightablePickMesh, PickHighlightParams},
    select::SelectablePickMesh,
};

use crate::{highlight::*, raycast::*, select::*};
use bevy::{
    prelude::*,
    render::camera::Camera,
    render::mesh::{VertexAttribute, VertexAttributeValues},
    render::pipeline::PrimitiveTopology,
    window::{CursorMoved, WindowId},
};
use std::collections::HashMap;

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PickState>()
            .init_resource::<PickHighlightParams>()
            .add_system(pick_mesh.system())
            .add_system(select_mesh.system())
            .add_system(build_rays.system())
            .add_system(pick_highlighting.system());
    }
}

pub struct PickState {
    ray_map: HashMap<PickingGroup, Ray3D>,
    ordered_pick_list_map: HashMap<PickingGroup, Vec<PickIntersection>>,
}

impl PickState {
    pub fn list(&self, group: PickingGroup) -> Option<&Vec<PickIntersection>> {
        self.ordered_pick_list_map.get(&group)
    }
    pub fn top(&self, group: PickingGroup) -> Option<&PickIntersection> {
        match self.ordered_pick_list_map.get(&group) {
            Some(list) => list.first(),
            None => None,
        }
    }
    pub fn top_all(&self) -> Vec<(&PickingGroup, &PickIntersection)> {
        let mut result = Vec::new();
        for (group, picklist) in self.ordered_pick_list_map.iter() {
            if let Some(pick) = picklist.first() {
                result.push((group, pick));
            }
        }
        result
    }
}

impl Default for PickState {
    fn default() -> Self {
        PickState {
            ray_map: HashMap::new(),
            ordered_pick_list_map: HashMap::new(),
        }
    }
}

/// Holds the entity associated with a mesh as well as it's computed intersection from a pick ray cast
#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub struct PickIntersection {
    entity: Entity,
    intersection: Ray3D,
    distance: f32,
}
impl PickIntersection {
    fn new(entity: Entity, intersection: Ray3D, distance: f32) -> Self {
        PickIntersection {
            entity,
            intersection,
            distance,
        }
    }
    /// Entity intersected with
    pub fn entity(&self) -> Entity {
        self.entity
    }
    /// Position vector describing the intersection position.
    pub fn position(&self) -> &Vec3 {
        self.intersection.origin()
    }
    /// Unit vector describing the normal of the intersected triangle.
    pub fn normal(&self) -> &Vec3 {
        self.intersection.direction()
    }
    /// Depth, distance from camera to intersection.
    pub fn distance(&self) -> f32 {
        self.distance
    }
}

/// Used to group pickable meshes with a camera into sets
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum PickingGroup {
    None,
    Group(usize),
}

impl Default for PickingGroup {
    fn default() -> Self {
        PickingGroup::Group(0)
    }
}

/// Marks an entity as pickable
#[derive(Debug)]
pub struct PickableMesh {
    group: Vec<PickingGroup>,
    bounding_sphere: Option<BoundingSphere>,
}

impl PickableMesh {
    pub fn new(picking_group: Vec<PickingGroup>) -> Self {
        PickableMesh {
            group: picking_group,
            bounding_sphere: None,
        }
    }
}

impl Default for PickableMesh {
    fn default() -> Self {
        PickableMesh {
            group: [PickingGroup::default()].into(),
            bounding_sphere: None,
        }
    }
}

#[derive(Debug)]
pub enum PickingMethod {
    Cursor(WindowId),
    ScreenSpace(Vec2),
    Center,
}

/// Marks an entity to be used for picking
pub struct PickingSource {
    group: PickingGroup,
    pick_method: PickingMethod,
    cursor_events: EventReader<CursorMoved>,
}

impl PickingSource {
    pub fn new(group: PickingGroup, pick_method: PickingMethod) -> Self {
        PickingSource {
            group,
            pick_method,
            ..Default::default()
        }
    }
    pub fn with_group(mut self, group: PickingGroup) -> Self {
        self.group = group;
        self
    }
    pub fn with_pick_method(mut self, pick_method: PickingMethod) -> Self {
        self.pick_method = pick_method;
        self
    }
}

impl Default for PickingSource {
    fn default() -> Self {
        PickingSource {
            group: PickingGroup::Group(0),
            pick_method: PickingMethod::Cursor(WindowId::primary()),
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
    mut pick_source_query: Query<(&mut PickingSource, &Transform, Entity)>,
    camera_query: Query<With<PickingSource, &Camera>>,
) {
    // Collect and calculate pick_ray from all cameras
    pick_state.ray_map.clear();

    for (mut pick_source, transform, entity) in &mut pick_source_query.iter() {
        let group_number = match pick_source.group {
            PickingGroup::Group(num) => num,
            PickingGroup::None => continue,
        };

        match pick_source.pick_method {
            PickingMethod::Cursor(window_id) => {
                let projection_matrix = match camera_query.get::<Camera>(entity) {
                    Ok(camera) => camera.projection_matrix,
                    Err(_) => panic!("The PickingSource in group {} has a {:?} but no associated Camera component", group_number, pick_source.pick_method),
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
                    None => continue,
                };

                // Get current screen size
                let window = windows.get(window_id).unwrap();
                let screen_size = Vec2::from([window.width as f32, window.height as f32]);

                // Normalized device coordinates (NDC) describes cursor position from (-1, -1, -1) to (1, 1, 1)
                let cursor_pos_ndc: Vec3 =
                    ((cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0])).extend(1.0);

                let camera_matrix = *transform.value();
                let (_, _, camera_position) = camera_matrix.to_scale_rotation_translation();

                let ndc_to_world: Mat4 = camera_matrix * projection_matrix.inverse();
                let cursor_position: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc);

                let ray_direction = cursor_position - camera_position;

                let pick_ray = Ray3D::new(camera_position, ray_direction);

                if pick_state
                    .ray_map
                    .insert(pick_source.group, pick_ray)
                    .is_some()
                {
                    panic!(
                        "Multiple PickingSources have been added to pick group: {}",
                        group_number
                    );
                }
            }
            PickingMethod::ScreenSpace(coordinates_ndc) => {
                let projection_matrix = match camera_query.get::<Camera>(entity) {
                    Ok(camera) => camera.projection_matrix,
                    Err(_) => panic!("The PickingSource in group {} has a {:?} but no associated Camera component", group_number, pick_source.pick_method),
                };
                let cursor_pos_ndc: Vec3 = coordinates_ndc.extend(1.0);
                let camera_matrix = *transform.value();
                let (_, _, camera_position) = camera_matrix.to_scale_rotation_translation();

                let ndc_to_world: Mat4 = camera_matrix * projection_matrix.inverse();
                let cursor_position: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc);

                let ray_direction = cursor_position - camera_position;

                let pick_ray = Ray3D::new(camera_position, ray_direction);

                if pick_state
                    .ray_map
                    .insert(pick_source.group, pick_ray)
                    .is_some()
                {
                    panic!(
                        "Multiple PickingSources have been added to pick group: {}",
                        group_number
                    );
                }
            }
            PickingMethod::Center => {
                let pick_position_ndc = Vec3::from([0.0, 0.0, 1.0]);
                let source_transform = *transform.value();
                let pick_position = source_transform.transform_point3(pick_position_ndc);

                let (_, _, source_origin) = source_transform.to_scale_rotation_translation();
                let ray_direction = pick_position - source_origin;

                let pick_ray = Ray3D::new(source_origin, ray_direction);

                if pick_state
                    .ray_map
                    .insert(pick_source.group, pick_ray)
                    .is_some()
                {
                    panic!(
                        "Multiple PickingSources have been added to pick group: {}",
                        group_number
                    );
                }
            }
        }
    }
}

fn pick_mesh(
    // Resources
    mut pick_state: ResMut<PickState>,
    meshes: Res<Assets<Mesh>>,
    // Queries
    mut mesh_query: Query<(&Handle<Mesh>, &Transform, &PickableMesh, Entity, &Draw)>,
) {
    // If there are no rays, then there is nothing to do here
    if pick_state.ray_map.is_empty() {
        return;
    } else {
        // TODO only clear out lists if the corresponding group has a ray
        pick_state.ordered_pick_list_map.clear();
    }

    // Iterate through each pickable mesh in the scene
    for (mesh_handle, transform, pickable, entity, draw) in &mut mesh_query.iter() {
        if !draw.is_visible {
            continue;
        }

        let pick_group = &pickable.group;

        // Check for a pick ray(s) in the group this mesh belongs to
        let mut pick_rays: Vec<(&PickingGroup, Ray3D)> = Vec::new();
        for group in pick_group.iter() {
            if let Some(ray) = pick_state.ray_map.get(group) {
                pick_rays.push((group, *ray));
            }
        }

        if pick_rays.is_empty() {
            continue;
        }

        // Use the mesh handle to get a reference to a mesh asset
        if let Some(mesh) = meshes.get(mesh_handle) {
            if mesh.primitive_topology != PrimitiveTopology::TriangleList {
                continue;
            }

            // Get the vertex positions from the mesh reference resolved from the mesh handle
            let vertex_positions: Vec<[f32; 3]> = mesh
                .attributes
                .iter()
                .filter(|attribute| attribute.name == VertexAttribute::POSITION)
                .filter_map(|attribute| match &attribute.values {
                    VertexAttributeValues::Float3(positions) => Some(positions.clone()),
                    _ => panic!("Unexpected vertex types in VertexAttribute::POSITION"),
                })
                .last()
                .unwrap();

            if let Some(indices) = &mesh.indices {
                // Iterate over the list of pick rays that belong to the same group as this mesh
                for (pick_group, pick_ray) in pick_rays {
                    // The ray cast can hit the same mesh many times, so we need to track which hit is
                    // closest to the camera, and record that.
                    let mut min_pick_distance = f32::MAX;

                    let mesh_to_world = transform.value();
                    let mut pick_intersection: Option<PickIntersection> = None;
                    // Now that we're in the vector of vertex indices, we want to look at the vertex
                    // positions for each triangle, so we'll take indices in chunks of three, where each
                    // chunk of three indices are references to the three vertices of a triangle.
                    for index in indices.chunks(3) {
                        // Make sure this chunk has 3 vertices to avoid a panic.
                        if index.len() != 3 {
                            break;
                        }
                        // Construct a triangle in world space using the mesh data
                        let mut vertices: [Vec3; 3] = [Vec3::zero(), Vec3::zero(), Vec3::zero()];
                        for i in 0..3 {
                            let vertex_pos_local = Vec3::from(vertex_positions[index[i] as usize]);
                            vertices[i] = mesh_to_world.transform_point3(vertex_pos_local)
                        }
                        let triangle = Triangle::from(vertices);
                        // Run the raycast on the ray and triangle
                        if let Some(intersection) = ray_triangle_intersection(
                            &pick_ray,
                            &triangle,
                            RaycastAlgorithm::default(),
                        ) {
                            let distance: f32 =
                                (*intersection.origin() - *pick_ray.origin()).length().abs();
                            if distance < min_pick_distance {
                                min_pick_distance = distance;
                                pick_intersection =
                                    Some(PickIntersection::new(entity, intersection, distance));
                            }
                        }
                    }
                    // Finished going through the current mesh, update pick states
                    if let Some(pick) = pick_intersection {
                        // Make sure the pick list map contains the key
                        match pick_state.ordered_pick_list_map.get_mut(pick_group) {
                            Some(list) => list.push(pick),
                            None => {
                                pick_state
                                    .ordered_pick_list_map
                                    .insert(*pick_group, Vec::from([pick]));
                            }
                        }
                    }
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
    for (_group, list) in pick_state.ordered_pick_list_map.iter_mut() {
        list.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}
