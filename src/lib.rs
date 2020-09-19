mod raycast;

use bevy::{
    prelude::*,
    render::camera::Camera,
    render::color::Color,
    render::mesh::{VertexAttribute, VertexAttributeValues},
    render::pipeline::PrimitiveTopology,
    window::CursorMoved,
};
use raycast::*;

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PickState>()
            .init_resource::<PickHighlightParams>()
            .add_system(pick_mesh.system())
            .add_system(select_mesh.system())
            .add_system(pick_highlighting.system());
    }
}

pub struct DebugPickingPlugin;
impl Plugin for DebugPickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_debug_cursor.system())
            .add_system(update_debug_cursor_position.system());
    }
}

pub struct PickState {
    cursor_event_reader: EventReader<CursorMoved>,
    ordered_pick_list: Vec<PickIntersection>,
}

impl PickState {
    pub fn list(&self) -> &Vec<PickIntersection> {
        &self.ordered_pick_list
    }
    pub fn top(&self) -> Option<&PickIntersection> {
        self.ordered_pick_list.first()
    }
}

impl Default for PickState {
    fn default() -> Self {
        PickState {
            cursor_event_reader: EventReader::default(),
            ordered_pick_list: Vec::new(),
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

#[derive(Debug)]
pub struct PickHighlightParams {
    hover_color: Color,
    selection_color: Color,
}

impl PickHighlightParams {
    pub fn hover_color_mut(&mut self) -> &mut Color {
        &mut self.hover_color
    }
    pub fn selection_color_mut(&mut self) -> &mut Color {
        &mut self.selection_color
    }
    pub fn set_hover_color(&mut self, color: Color) {
        self.hover_color = color;
    }
    pub fn set_selection_color(&mut self, color: Color) {
        self.selection_color = color;
    }
}

impl Default for PickHighlightParams {
    fn default() -> Self {
        PickHighlightParams {
            hover_color: Color::rgb(0.3, 0.5, 0.8),
            selection_color: Color::rgb(0.3, 0.8, 0.5),
        }
    }
}

/// Marks an entity as pickable
#[derive(Debug)]
pub struct PickableMesh {
    camera_entity: Entity,
    bounding_sphere: Option<BoundingSphere>,
}

impl PickableMesh {
    pub fn new(camera_entity: Entity) -> Self {
        PickableMesh {
            camera_entity,
            bounding_sphere: None,
        }
    }
}

/// Meshes with `SelectableMesh` will have selection state managed
#[derive(Debug)]
pub struct SelectablePickMesh {
    selected: bool,
}

impl SelectablePickMesh {
    pub fn new() -> Self {
        SelectablePickMesh::default()
    }
    pub fn selected(&self) -> bool {
        self.selected
    }
}

impl Default for SelectablePickMesh {
    fn default() -> Self {
        SelectablePickMesh { selected: false }
    }
}

/// Meshes with `HighlightablePickMesh` will be highlighted when hovered over.
/// If the mesh also has the `SelectablePickMesh` component, it will highlight when selected.
#[derive(Debug)]
pub struct HighlightablePickMesh {
    // Stores the initial color of the mesh material prior to selecting/hovering
    initial_color: Option<Color>,
}

impl HighlightablePickMesh {
    pub fn new() -> Self {
        HighlightablePickMesh::default()
    }
}

impl Default for HighlightablePickMesh {
    fn default() -> Self {
        HighlightablePickMesh {
            initial_color: None,
        }
    }
}

struct DebugCursor;

struct DebugCursorMesh;

/// Updates the 3d cursor to be in the pointed world coordinates
fn update_debug_cursor_position(
    pick_state: Res<PickState>,
    mut query: Query<With<DebugCursor, &mut Transform>>,
    mut visibility_query: Query<With<DebugCursorMesh, &mut Draw>>,
) {
    // Set the cursor translation to the top pick's world coordinates
    if let Some(top_pick) = pick_state.top() {
        let position = top_pick.position();
        let normal = top_pick.normal();
        let up = Vec3::from([0.0, 1.0, 0.0]);
        let axis = up.cross(*normal).normalize();
        let angle = up.dot(*normal).acos();
        let epsilon = 0.0001;
        let new_rotation = if angle.abs() > epsilon {
            Quat::from_axis_angle(axis, angle)
        } else {
            Quat::default()
        };
        let transform_new = Mat4::from_rotation_translation(new_rotation, *position);
        for mut transform in &mut query.iter() {
            *transform.value_mut() = transform_new;
        }
        for mut draw in &mut visibility_query.iter() {
            draw.is_visible = true;
        }
    } else {
        for mut draw in &mut visibility_query.iter() {
            draw.is_visible = false;
        }
    }
}

/// Start up system to create 3d Debug cursor
fn setup_debug_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_matl = materials.add(StandardMaterial {
        albedo: Color::rgb(0.0, 1.0, 0.0),
        shaded: false,
        ..Default::default()
    });
    let cube_size = 0.02;
    let cube_tail_scale = 20.0;
    let ball_size = 0.08;
    commands
        // cursor
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                subdivisions: 4,
                radius: ball_size,
            })),
            material: debug_matl,
            ..Default::default()
        })
        .with_children(|parent| {
            // child cube
            parent
                .spawn(PbrComponents {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: cube_size })),
                    material: debug_matl,
                    transform: Transform::from_non_uniform_scale(Vec3::from([
                        1.0,
                        cube_tail_scale,
                        1.0,
                    ]))
                    .with_translation(Vec3::new(
                        0.0,
                        cube_size * cube_tail_scale,
                        0.0,
                    )),
                    ..Default::default()
                })
                .with(DebugCursorMesh);
        })
        .with(DebugCursor)
        .with(DebugCursorMesh);
}

/// Given the current selected and hovered meshes and provided materials, update the meshes with the
/// appropriate materials.
fn pick_highlighting(
    // Resources
    pick_state: Res<PickState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    highlight_params: Res<PickHighlightParams>,
    // Queries
    mut query_picked: Query<(
        &mut HighlightablePickMesh,
        &PickableMesh,
        &Handle<StandardMaterial>,
        Entity,
    )>,
    mut query_selected: Query<(
        &mut HighlightablePickMesh,
        &SelectablePickMesh,
        &Handle<StandardMaterial>,
    )>,
    mut query_selectables: Query<&SelectablePickMesh>,
) {
    // Query selectable entities that have changed
    for (mut highlightable, selectable, material_handle) in &mut query_selected.iter() {
        let current_color = &mut materials.get_mut(material_handle).unwrap().albedo;
        let initial_color = match highlightable.initial_color {
            None => {
                highlightable.initial_color = Some(*current_color);
                *current_color
            }
            Some(color) => color,
        };
        if selectable.selected {
            *current_color = highlight_params.selection_color;
        } else {
            *current_color = initial_color;
        }
    }

    // Query highlightable entities that have changed
    for (mut highlightable, _pickable, material_handle, entity) in &mut query_picked.iter() {
        let current_color = &mut materials.get_mut(material_handle).unwrap().albedo;
        let initial_color = match highlightable.initial_color {
            None => {
                highlightable.initial_color = Some(*current_color);
                *current_color
            }
            Some(color) => color,
        };
        let mut topmost = false;
        if let Some(pick_depth) = pick_state.top() {
            topmost = pick_depth.entity == entity;
        }
        if topmost {
            *current_color = highlight_params.hover_color;
        } else if let Ok(mut query) = query_selectables.entity(entity) {
            if let Some(selectable) = query.get() {
                if selectable.selected {
                    *current_color = highlight_params.selection_color;
                } else {
                    *current_color = initial_color;
                }
            }
        } else {
            *current_color = initial_color;
        }
    }
}

/// Given the currently hovered mesh, checks for a user click and if detected, sets the selected
/// field in the entity's component to true.
fn select_mesh(
    // Resources
    pick_state: Res<PickState>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    // Queries
    mut query: Query<&mut SelectablePickMesh>,
) {
    if mouse_button_inputs.just_pressed(MouseButton::Left) {
        // Deselect everything
        for mut selectable in &mut query.iter() {
            selectable.selected = false;
        }

        if let Some(pick_depth) = pick_state.top() {
            if let Ok(mut top_mesh) = query.get_mut::<SelectablePickMesh>(pick_depth.entity) {
                top_mesh.selected = true;
            }
        }
    }
}

fn pick_mesh(
    // Resources
    mut pick_state: ResMut<PickState>,
    cursor: Res<Events<CursorMoved>>,
    meshes: Res<Assets<Mesh>>,
    windows: Res<Windows>,
    // Queries
    mut mesh_query: Query<(&Handle<Mesh>, &Transform, &PickableMesh, Entity, &Draw)>,
    mut camera_query: Query<(&Transform, &Camera)>,
) {
    // Get the cursor position
    let cursor_pos_screen: Vec2 = match pick_state.cursor_event_reader.latest(&cursor) {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    // Get current screen size
    let window = windows.get_primary().unwrap();
    let screen_size = Vec2::from([window.width as f32, window.height as f32]);

    // Normalized device coordinates (NDC) describes cursor position from (-1, -1, -1) to (1, 1, 1)
    let cursor_pos_ndc: Vec3 =
        ((cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0])).extend(1.0);

    // Get the view transform and projection matrix from the camera
    let mut camera_matrix = Mat4::zero();
    let mut projection_matrix = Mat4::zero();
    for (transform, camera) in &mut camera_query.iter() {
        camera_matrix = *transform.value();
        projection_matrix = camera.projection_matrix;
    }
    let (_, _, camera_position) = camera_matrix.to_scale_rotation_translation();

    let ndc_to_world: Mat4 = camera_matrix * projection_matrix.inverse();
    let cursor_position: Vec3 = ndc_to_world.transform_point3(cursor_pos_ndc);

    let ray_direction = cursor_position - camera_position;

    let pick_ray = Ray3D::new(camera_position, ray_direction);

    // After initial checks completed, clear the pick list
    pick_state.ordered_pick_list.clear();

    // Iterate through each pickable mesh in the scene
    for (mesh_handle, transform, _pickable, entity, draw) in &mut mesh_query.iter() {
        if !draw.is_visible {
            continue;
        }

        // Use the mesh handle to get a reference to a mesh asset
        if let Some(mesh) = meshes.get(mesh_handle) {
            if mesh.primitive_topology != PrimitiveTopology::TriangleList {
                continue;
            }

            // The ray cast can hit the same mesh many times, so we need to track which hit is
            // closest to the camera, and record that.
            let mut min_pick_distance = f32::MAX;

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
                    if let Some(intersection) =
                        ray_triangle_intersection(&pick_ray, &triangle, RaycastAlgorithm::default())
                    {
                        let distance: f32 =
                            (*intersection.origin() - camera_position).length().abs();
                        if distance < min_pick_distance {
                            min_pick_distance = distance;
                            pick_intersection =
                                Some(PickIntersection::new(entity, intersection, distance));
                        }
                    }
                }
                // Finished going through the current mesh, update pick states
                if let Some(pick) = pick_intersection {
                    pick_state.ordered_pick_list.push(pick);
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
    pick_state.ordered_pick_list.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
