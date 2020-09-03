use bevy::{
    prelude::*,
    render::camera::Camera,
    render::mesh::{VertexAttribute, VertexAttributeValues},
    render::pipeline::PrimitiveTopology,
    render::color::Color,
    window::CursorMoved,
};

pub struct PickingPlugin;
impl Plugin for PickingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PickState>()
            .add_system(pick_mesh.system())
            //.add_system(select_mesh.system())
            //.add_system(pick_highlighting.system())
            ;
    }
}

pub struct PickState {
    cursor_event_reader: EventReader<CursorMoved>,
    ordered_pick_list: Vec<PickDepth>,
}

impl PickState {
    pub fn list(&self) -> &Vec<PickDepth> {
        &self.ordered_pick_list
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

/// Holds the entity associated with a mesh as well as it's computed depth from a pick ray cast
#[derive(Debug, PartialOrd, PartialEq)]
pub struct PickDepth {
    entity: Entity,
    ndc_depth: f32,
}
impl PickDepth {
    fn new(entity: Entity, ndc_depth: f32) -> Self {
        PickDepth{
            entity,
            ndc_depth,
        }
    }
}

/// Holds a list of selected meshes by handle
#[derive(Debug)]
pub struct PickSelectionState {
    selected_next: Vec<Handle<Mesh>>,
    selected_previous: Vec<Handle<Mesh>>,
}

#[derive(Debug)]

pub struct PickHighlightState {
    hover_color: Color,
    selection_color: Color,
    // Used to track items highlighted in the last frame. This is compared against the
    // current pick list to see which items need to have their highligh effect removed.
    // TODO: use a Vec to *pop* items off when comparing, to reduce list size as items are
    // matched? Instead of iterating over two vecs which remain the same size.
    hovered_previous: Option<Handle<Mesh>>,
}

impl PickHighlightState {
    fn set_hover_color(&mut self, color: Color) {
        self.hover_color = color;
    }
    fn set_selection_color(&mut self, color: Color) {
        self.selection_color = color;
    }
}

impl Default for PickHighlightState {
    fn default() -> Self {
        PickHighlightState {
            hover_color: Color::rgb(0.3, 0.5, 0.8),
            selection_color: Color::rgb(0.3, 0.8, 0.5),
            hovered_previous: None,
        }
    }
}

/// Marks an entity as pickable
#[derive(Debug)]
pub struct PickableMesh {
    bounding_sphere: BoundSphere,
}

impl PickableMesh {
    pub fn new(parent_mesh: &Mesh) -> Self {
        PickableMesh {
            bounding_sphere: BoundSphere::from(parent_mesh),
        }
    }
    fn update_ndc_bounding_circle() {

    }
}

/// Meshes with `SelectableMesh` will have selection state managed
#[derive(Debug)]
pub struct SelectablePickMesh;

/// Meshes with `HighlightablePickMesh` will be highlighted when hovered over. If the mesh also has
/// the `SelectablePickMesh` component, it will highlight when selected.
#[derive(Debug)]
pub struct HighlightablePickMesh {
    // Stores the initial color of the mesh material prior to selecting/hovering
    initial_color: Color,
}

impl HighlightablePickMesh {
    pub fn new(mesh_material: &StandardMaterial) -> Self {
        HighlightablePickMesh {
            initial_color: mesh_material.albedo,
        }
    }
}

// How to handle bounding spheres?
// Need to update any time the mesh or its transform changes. Initial query of points remains valid
// as long as the mesh and scale does not change. Ensure the sphere is centered on the mesh center,
// so that changes to rotation do not affect the bounding sphere definition, and scale only affects
// the radius. The mesh query should compute the distance of each point from the origin, and store
// the maximum value as the radius.
// In summary:
// 1. query mesh to determine radius via max
// 2. on setup, determine the NDC coordinates of the sphere using the entity's mesh translation,
//    scale(radius), make sure to use the NDC z-coord to divide by w on the radius for perspective
//    WARNING!!!!: rectilinear perpective warp means bounding sphere will be elliptical(?) need to add
//    some buffer to ndc radius to account for this as a function of FOV and distance from the ndc 
//    origin. This is a conic section. http://shaunlebron.github.io/visualizing-projections/
//    perspective_scaling = sec(arctan(x*tan(b/2))) where b = fov in radians
//    or sqrt(c*x^2+1) where c has some nonlinear relationship to fov. c = tan(b/2) is close but not
//    identical: https://www.desmos.com/calculator/v0sf4wota5
// 3. If the mesh changes, recompute the mesh_radius and ndc_def
// 4. If the camera or mesh transforms change, update the ndc_def

/// Defines a bounding sphere with a center point coordinate and a radius, used for picking
#[derive(Debug)]
struct BoundSphere {
    mesh_radius: f32,
    ndc_def: Option<NdcBoundingCircle>,
}

impl From<&Mesh> for BoundSphere {
    fn from(mesh: &Mesh) -> Self {
        let mut mesh_radius = 0f32;
        if mesh.primitive_topology != PrimitiveTopology::TriangleList {
            panic!("Non-TriangleList mesh supplied for bounding sphere generation")
        }
        let mut vertex_positions = Vec::new();
        for attribute in mesh.attributes.iter() {
            if attribute.name == VertexAttribute::POSITION {
                vertex_positions = match &attribute.values {
                    VertexAttributeValues::Float3(positions) => positions.clone(),
                    _ => panic!("Unexpected vertex types in VertexAttribute::POSITION"),
                };
            }
        }
        if let Some(indices) = &mesh.indices {
            for index in indices.iter() {
                mesh_radius = mesh_radius.max(Vec3::from(vertex_positions[*index as usize]).length());
            }
        }
        BoundSphere {
            mesh_radius,
            ndc_def: None,
        }
    }
}

/// Created from a BoundSphere, this represents a circle that bounds the entity's mesh when the
/// bounding sphere is projected onto the screen. Note this is not as simple as transforming the
/// sphere's origin into ndc and copying the radius. Due to rectillinear projection, the sphere
/// will be projected onto the screen as an ellipse if it is not perfectly centered at 0,0 in ndc.
#[derive(Debug)]
struct NdcBoundingCircle {
    center: Vec2,
    radius: f32,
}


/*
/// Given the current selected and hovered meshes and provided materials, update the meshes with the
/// appropriate materials.
fn pick_highlighting(
    // Resources
    pick_state: ResMut<PickingState>,
    materials: Res<Assets<StandardMaterial>>,
    // Queries
    mut query: Query<(
        &mut SelectableMesh,
        &mut Handle<StandardMaterial>,
        &Handle<Mesh>,
    )>,
) {
    if pick_state.hovered != pick_state.hovered_previous {
        println!(
            "Hover state change from {:?} to {:?}",
            pick_state.hovered_previous, pick_state.hovered
        );
    }
    if pick_state.selected != pick_state.selected_previous {
        println!(
            "Selection state change from {:?} to {:?}",
            pick_state.selected_previous, pick_state.selected
        );
    }
    for (mut selectable, mut matl_handle, mesh_handle) in &mut query.iter() {
        if let None = selectable.material_default {
            selectable.material_default = Some(*matl_handle);
        }

        if pick_state.selected != pick_state.selected_previous {
            if let Some(previous) = pick_state.selected_previous {
                if *mesh_handle == previous {
                    match selectable.material_default {
                        Some(default_matl) => {
                            *matl_handle = default_matl.clone();
                        }
                        None => panic!("Default material not set for previously selected mesh"),
                    }
                }
            }
            if let Some(selected) = pick_state.selected {
                if *mesh_handle == selected {
                    *matl_handle = pick_state.selected_material.clone();
                }
            }
        }

        if pick_state.hovered != pick_state.hovered_previous {
            if let Some(previous) = pick_state.hovered_previous {
                if *mesh_handle == previous {
                    match selectable.material_default {
                        Some(default_matl) => {
                            //println!("Hover material: {:?}", selectable.material_default);
                            *matl_handle = default_matl.clone();
                        }
                        None => panic!("Default material not set for previously hovered mesh"),
                    }
                }
            }
            if pick_state.selected == Some(*mesh_handle) {
                *matl_handle = pick_state.selected_material.clone();
            }
            if let Some(hovered) = pick_state.hovered {
                if *mesh_handle == hovered {
                    *matl_handle = pick_state.hovered_material.clone();
                }
            }
        }
        if pick_state.hovered != pick_state.hovered_previous ||
            pick_state.selected != pick_state.selected_previous
        {
            println!("Material: {:?}", materials.get(&*matl_handle).unwrap().albedo);
        }
    }
}
*/

/*
/// Given the currently hovered mesh, checks for a user click and if detected, sets the selected
/// mesh in the MousePicking state resource.
fn select_mesh(
    // Resources
    mut pick_state: ResMut<PickState>,
    mouse_button_inputs: Res<Input<MouseButton>>,
    // Queries
    mut query: Query<(&Handle<Mesh>, &PickableMesh)>,
) {
    let mut selection_made = false;
    let mut pick_stack = pick_state.ordered_pick_list;
    query.get(entity)
    for (mesh_handle, _selectable) in &mut query.iter() {

        if let Some(hovered) = pick_state.hovered {
            // If the current mesh is the one being hovered over, and the left mouse button is
            // down, set the current mesh to selected.
            if *mesh_handle == hovered && mouse_button_inputs.pressed(MouseButton::Left) {
                pick_state.selected_previous = pick_state.selected;
                // Set the current mesh as the selected mesh.
                pick_state.selected = Some(*mesh_handle);
                selection_made = true;
            }
        }
    }
    // If nothing is being hovered and the user clicks, deselect the current mesh.
    if pick_state.hovered == None
        && mouse_button_inputs.pressed(MouseButton::Left)
        && pick_state.selected != None
    {
        pick_state.selected_previous = pick_state.selected;
        pick_state.selected = None;
        selection_made = true;
    }
    if !selection_made {
        pick_state.selected_previous = pick_state.selected;
    }
}
*/

fn pick_mesh(
    // Resources
    mut pick_state: ResMut<PickState>,
    cursor: Res<Events<CursorMoved>>,
    meshes: Res<Assets<Mesh>>,
    windows: Res<Windows>,
    // Queries
    mut mesh_query: Query<(&Handle<Mesh>, &Transform, &PickableMesh, &Entity)>,
    mut camera_query: Query<(&Transform, &Camera)>,
) {
    pick_state.ordered_pick_list.clear();
    
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
    for (mesh_handle, transform, _selectable, entity) in &mut mesh_query.iter() {
        // Use the mesh handle to get a reference to a mesh asset
        if let Some(mesh) = meshes.get(mesh_handle) {
            if mesh.primitive_topology != PrimitiveTopology::TriangleList {
                continue;
            }

            // The ray cast can hit the same mesh many times, so we need to track which hit is
            // closest to the camera, and record that.
            let mut hit_depth = f32::MAX;

            // We need to transform the mesh vertices' positions from the mesh space to the world
            // space using the mesh's transform, move it to the camera's space using the view
            // matrix (camera.inverse), and finally, apply the projection matrix. Because column
            // matrices are evaluated right to left, we have to order it correctly:
            let combined_transform = view_matrix * transform.value;

            // Get the vertex positions from the mesh reference resolved from the mesh handle
            let vertex_positions: Vec<[f32; 3]> = mesh.attributes.iter()
                .filter(|attribute| attribute.name == VertexAttribute::POSITION)
                .filter_map(|attribute| match &attribute.values {
                    VertexAttributeValues::Float3(positions) => Some(positions.clone()),
                    _ => panic!("Unexpected vertex types in VertexAttribute::POSITION"),
                }).last().unwrap();

            /*
            let mut vertex_positions = Vec::new();
            for attribute in mesh.attributes.iter() {
                if attribute.name == VertexAttribute::POSITION {
                    vertex_positions = match &attribute.values {
                        VertexAttributeValues::Float3(positions) => positions.clone(),
                        _ => panic!("Unexpected vertex types in VertexAttribute::POSITION"),
                    };
                }
            }*/

            // We have everything set up, now we can jump into the mesh's list of indices and
            // check triangles for cursor intersection.
            if let Some(indices) = &mesh.indices {
                let mut hit_found = false;
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
                            hit_found = true;
                            if  triangle[0].z() < hit_depth {
                                hit_depth = triangle[0].z();
                                //println!("hit depth: {}", hit_depth);
                                // if the hovered mesh has changed, update the pick state
                            }
                        }
                    }
                }

                if hit_found {
                    pick_state.ordered_pick_list.push(
                        PickDepth::new(*entity, hit_depth)
                    );
                    println!("Mesh Pick: {:?}", pick_state.ordered_pick_list.last());
                }

            } else {
                panic!(
                    "No index matrix found in mesh {:?}\n{:?}",
                    mesh_handle, mesh
                );
            }
        }
    }

    // Sort the pick list
    pick_state
        .ordered_pick_list
        .sort_by(|a, b| a.partial_cmp(b)
        .unwrap_or(std::cmp::Ordering::Equal));
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
