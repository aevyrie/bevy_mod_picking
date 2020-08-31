use bevy::{
    prelude::*,
    render::camera::Camera,
    render::mesh::{VertexAttribute, VertexAttributeValues},
    render::pipeline::PrimitiveTopology,
    window::CursorMoved,
};

pub struct ModPicking;
impl Plugin for ModPicking {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<MousePicking>()
            .add_system(cursor_pick.system())
            .add_system(pick_selection.system())
            .add_system(pick_highlighting.system());
    }
}

pub struct MousePicking {
    // Collects cursor position on screen in x/y
    cursor_event_reader: EventReader<CursorMoved>,
    pub hovered_material: Handle<StandardMaterial>,
    pub selected_material: Handle<StandardMaterial>,
    hovered: Option<Handle<Mesh>>,
    hovered_previous: Option<Handle<Mesh>>,
    selected: Option<Handle<Mesh>>,
    selected_previous: Option<Handle<Mesh>>,
}

impl Default for MousePicking {
    fn default() -> Self {
        MousePicking {
            cursor_event_reader: EventReader::default(),
            hovered_material: Handle::default(),
            selected_material: Handle::default(),
            hovered: None,
            hovered_previous: None,
            selected: None,
            selected_previous: None,
        }
    }
}

/// Marks an entity as selectable for picking
pub struct Selectable {
    // Stores the base material of a previously selected/hovered mesh
    material_default: Option<Handle<StandardMaterial>>,
}

impl Default for Selectable {
    fn default() -> Self {
        Selectable {
            material_default: None,
        }
    }
}

/// Given the current selected and hovered meshes and provided materials, update the meshes with the
/// appropriate materials.
fn pick_highlighting(
    // Resources
    mut pick_state: ResMut<MousePicking>,
    // Queries
    mut query: Query<(&mut Selectable, &mut Handle<StandardMaterial>, &Handle<Mesh>)>,
) {
    for (mut selectable, mut matl_handle, mesh_handle) in &mut query.iter() {
        if let Some(hovered) = pick_state.hovered {
            if *mesh_handle == hovered {
                // If the current mesh hasn't stored it's default material, we need to
                // do that now so it can be reset once it is no longer being hovered.
                if let None = selectable.material_default {
                    selectable.material_default = Some(*matl_handle);
                }
            }
        }

        // MousePicking selected_previous is only filled if the selected item changed. If so, the
        // selected_previous material needs to be reset to its default material.
        if let Some(previous) = pick_state.selected_previous {
            if *mesh_handle == previous {
                match selectable.material_default {
                    Some(default_matl) => {
                        *matl_handle = default_matl;
                    }
                    None => panic!("Default material not set for previously selected mesh"),
                }
            } else if let Some(selected) = pick_state.selected {
                if *mesh_handle == selected {
                    *matl_handle = pick_state.selected_material;
                }
            }
        } else {
            if let Some(selected) = pick_state.selected {
                if *mesh_handle == selected {
                    *matl_handle = pick_state.selected_material;
                }
            }
        }
        // MousePicking hovered_previous is only filled if the hovered item has changed. If so, the
        // hovered_previous material needs to be reset to its default material.
        if let Some(previous) = pick_state.hovered_previous {
            if *mesh_handle == previous {
                match selectable.material_default {
                    Some(default_matl) => {
                        *matl_handle = default_matl;
                    }
                    None => panic!("Default material not set for previously selected mesh"),
                }
            } else if let Some(hovered) = pick_state.hovered {
                if *mesh_handle == hovered {
                    *matl_handle = pick_state.hovered_material;
                }
            }
        } else {
            if let Some(hovered) = pick_state.hovered {
                if *mesh_handle == hovered {
                    *matl_handle = pick_state.hovered_material;
                }
            }
        }
    }
    // The picking highlighting process is complete, the `_previous` variables can now be reset, as
    // they are only used to inform the system that their materials need to be set to default.
    pick_state.hovered_previous = None;
    pick_state.selected_previous = None;
}

/// Given the currently hovered mesh, checks for a user click and if detected, sets the selected
/// mesh in the MousePicking state resource.
fn pick_selection(
    // Resources
    mut pick_state: ResMut<MousePicking>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    // Queries
    mut query: Query<&Handle<Mesh>>,
) {
    for mesh_handle in &mut query.iter() {
        if let Some(hovered) = pick_state.hovered {
            // If the current mesh is the one being hovered over, and the left mouse button is
            // down, set the current mesh to selected.
            if *mesh_handle == hovered && mouse_button_inputs.pressed(MouseButton::Left) {
                // If there is a previously selected mesh, we need to clear it.
                pick_state.selected_previous = pick_state.selected;
                // Set the current mesh as the selected mesh.
                pick_state.selected = Some(*mesh_handle)
            }
        }
    }
    // If nothing is being hovered and the user clicks, deselect the current mesh.
    if pick_state.hovered == None && mouse_button_inputs.pressed(MouseButton::Left) {
        pick_state.selected_previous = pick_state.selected;
        pick_state.selected = None;
    }
}

fn cursor_pick(
    // Resources
    mut pick_state: ResMut<MousePicking>,
    cursor: Res<Events<CursorMoved>>,
    meshes: Res<Assets<Mesh>>,
    windows: Res<Windows>,
    // Queries
    mut mesh_query: Query<(
        &mut Selectable,
        &Handle<Mesh>,
        &mut Handle<StandardMaterial>,
        &Transform,
    )>,
    mut camera_query: Query<(&Transform, &Camera)>,
) {
    // To start, assume noting is being hovered.
    pick_state.hovered = None;

    // Get the cursor position
    let cursor_pos_screen: Vec2 = match pick_state.cursor_event_reader.latest(&cursor) {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    // Get current screen size
    let window = windows.get_primary().unwrap();
    let screen_size = Vec2::from([window.width as f32, window.height as f32]);

    // Normalized device coordinates (NDC) describes cursor position from (-1, -1) to (1, 1)
    let cursor_pos_ndc: Vec2 = (cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);

    // Get the view transform and projection matrix from the camera
    let mut view_matrix = Mat4::zero();
    let mut projection_matrix = Mat4::zero();
    for (transform, camera) in &mut camera_query.iter() {
        view_matrix = transform.value.inverse();
        projection_matrix = camera.projection_matrix;
    }

    // Iterate through each selectable mesh in the scene
    'mesh_loop: for (mut selectable, mesh_handle, matl_handle, transform) in &mut mesh_query.iter()
    {
        // Use the mesh handle to get a reference to a mesh asset
        if let Some(mesh) = meshes.get(mesh_handle) {
            if mesh.primitive_topology != PrimitiveTopology::TriangleList {
                continue;
            }
            // We need to transform the mesh vertices' positions from the mesh space to the world
            // space using the mesh's transform, move it to the camera's space using the view
            // matrix (camera.inverse), and finally, apply the projection matrix. Because column
            // matrices are evaluated right to left, we have to order it correctly:
            let combined_transform = view_matrix * transform.value;

            // Get the vertex positions from the mesh reference resolved from the mesh handle
            let mut vertex_positions = Vec::new();
            for attribute in mesh.attributes.iter() {
                if attribute.name == VertexAttribute::POSITION {
                    vertex_positions = match &attribute.values {
                        VertexAttributeValues::Float3(positions) => positions.clone(),
                        _ => panic!("Unexpected vertex types in VertexAttribute::POSITION"),
                    };
                }
            }

            // We have everything set up, now we can jump into the mesh's list of indices and
            // check triangles for cursor intersection.
            if let Some(indices) = &mesh.indices {
                // Now that we're in the vector of vertex indices, we want to look at the vertex
                // positions for each triangle, so we'll take indices in chunks of three, where each
                // chunk of three indices are references to the three vertices of a triangle.
                for index in indices.chunks(3) {
                    // Make sure this chunk has 3 vertices to avoid a panic.
                    if index.len() == 3 {
                        // Set up an empty container for triangle vertices
                        let mut triangle: [Vec3; 3] = [Vec3::zero(), Vec3::zero(), Vec3::zero()];
                        // We can now grab the position of each vertex in the triangle using the
                        // indices pointing into the position vector. These positions are relative
                        // to the coordinate system of the mesh the vertex/triangle belongs to. To
                        // test if the triangle is being hovered over, we need to convert this to
                        // NDC (normalized device coordinates)
                        for i in 0..3 {
                            // Get the raw vertex position using the index
                            let mut vertex_pos = Vec3::from(vertex_positions[index[i] as usize]);
                            // Transform the vertex to world space with the mesh transform, then
                            // into camera space with the view transform.
                            vertex_pos = combined_transform.transform_point3(vertex_pos);
                            // This next part seems to be a bug with glam - it should do the divide
                            // by w perspective math for us, instead we have to do it manually.
                            // `glam` PR https://github.com/bitshifter/glam-rs/pull/75/files
                            let transformed = projection_matrix.mul_vec4(vertex_pos.extend(1.0));
                            let w = transformed.w();
                            triangle[i] = Vec3::from(transformed.truncate() / w);
                        }
                        if point_in_tri(
                            &cursor_pos_ndc,
                            &Vec2::new(triangle[0].x(), triangle[0].y()),
                            &Vec2::new(triangle[1].x(), triangle[1].y()),
                            &Vec2::new(triangle[2].x(), triangle[2].y()),
                        ) {
                            // if the hovered mesh has changed, update the pick state
                            let current_hovered_mesh = Some(*mesh_handle);
                            if pick_state.hovered != current_hovered_mesh {
                                pick_state.hovered_previous = pick_state.hovered;
                                pick_state.hovered = current_hovered_mesh
                            } else {
                                pick_state.hovered_previous = None;
                            }
                            println!("HIT! {}", mesh_handle.id.0);
                            continue 'mesh_loop;
                        }
                    }
                }
            } else {
                panic!(
                    "No index matrix found in mesh {:?}\n{:?}",
                    mesh_handle, mesh
                );
            }
            println!("No collision in {}", mesh_handle.id.0);
        }
    }
}

/// Compute the area of a triangle given 2D vertex coordinates, "/2" removed to save an operation
fn double_tri_area(a: &Vec2, b: &Vec2, c: &Vec2) -> f32 {
    f32::abs(a.x() * (b.y() - c.y()) + b.x() * (c.y() - a.y()) + c.x() * (a.y() - b.y()))
}

/// Checks if a point is inside a triangle by comparing the summed areas of the triangles, the point
/// is inside the triangle if the areas are equal. An epsilon is used due to floating point error.
/// Todo: barycentric method
fn point_in_tri(p: &Vec2, a: &Vec2, b: &Vec2, c: &Vec2) -> bool {
    let area = double_tri_area(a, b, c);
    let pab = double_tri_area(p, a, b);
    let pac = double_tri_area(p, a, c);
    let pbc = double_tri_area(p, b, c);
    let area_tris = pab + pac + pbc;
    let epsilon = 0.000001;
    //println!("{:.3}  {:.3}", area, area_tris);
    f32::abs(area - area_tris) < epsilon
}
