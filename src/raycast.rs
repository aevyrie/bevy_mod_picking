use bevy::{
    prelude::*,
    render::mesh::{VertexAttribute, VertexAttributeValues},
    render::pipeline::PrimitiveTopology,
};

pub use rays::*;

pub mod rays {

    use bevy::prelude::*;

    #[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
    pub struct Ray3D {
        origin: Vec3,
        direction: Vec3,
    }

    impl Ray3D {
        pub fn new(origin: Vec3, direction: Vec3) -> Self {
            Ray3D {
                origin,
                direction: direction.normalize(),
            }
        }
        /// Position vector describing the ray origin
        pub fn origin(&self) -> &Vec3 {
            &self.origin
        }
        /// Unit vector describing the ray direction
        pub fn direction(&self) -> &Vec3 {
            &self.direction
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
}

impl Triangle {
    /// Returns a tuple of vertices, useful for variable assignment
    pub fn vertices(&self) -> (Vec3, Vec3, Vec3) {
        (self.v0, self.v1, self.v2)
    }
}

impl From<(Vec3, Vec3, Vec3)> for Triangle {
    fn from(vertices: (Vec3, Vec3, Vec3)) -> Self {
        Triangle {
            v0: vertices.0,
            v1: vertices.1,
            v2: vertices.2,
        }
    }
}

impl From<Vec<Vec3>> for Triangle {
    fn from(vertices: Vec<Vec3>) -> Self {
        Triangle {
            v0: *vertices.get(0).unwrap(),
            v1: *vertices.get(1).unwrap(),
            v2: *vertices.get(2).unwrap(),
        }
    }
}

impl From<[Vec3; 3]> for Triangle {
    fn from(vertices: [Vec3; 3]) -> Self {
        Triangle {
            v0: vertices[0],
            v1: vertices[1],
            v2: vertices[2],
        }
    }
}

/// Defines a bounding sphere with a center point coordinate and a radius
#[derive(Debug)]
pub struct BoundingSphere {
    mesh_radius: f32,
    transformed_radius: Option<f32>,
}

impl From<&Mesh> for BoundingSphere {
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
                mesh_radius =
                    mesh_radius.max(Vec3::from(vertex_positions[*index as usize]).length());
            }
        }
        BoundingSphere {
            mesh_radius,
            transformed_radius: None,
        }
    }
}

pub enum RaycastAlgorithm {
    Geometric,
    MollerTrumbore(Backfaces),
}

impl Default for RaycastAlgorithm {
    fn default() -> Self {
        RaycastAlgorithm::MollerTrumbore(Backfaces::Cull)
    }
}

pub enum Backfaces {
    Cull,
    Include,
}

/// Takes a ray and triangle and computes the intersection and normal
pub fn ray_triangle_intersection(
    ray: &Ray3D,
    triangle: &Triangle,
    algorithm: RaycastAlgorithm,
) -> Option<Ray3D> {
    match algorithm {
        RaycastAlgorithm::Geometric => raycast_geometric(ray, triangle),
        RaycastAlgorithm::MollerTrumbore(backface_culling) => {
            raycast_moller_trumbore(ray, triangle, backface_culling)
        }
    }
}

/// Implementation of the Möller-Trumbore ray-triangle intersection test
pub fn raycast_moller_trumbore(
    ray: &Ray3D,
    triangle: &Triangle,
    backface_culling: Backfaces,
) -> Option<Ray3D> {
    // Source: https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection
    let epsilon: f32 = 0.000001;
    let vector_v0_to_v1: Vec3 = triangle.v1 - triangle.v0;
    let vector_v0_to_v2: Vec3 = triangle.v2 - triangle.v0;
    let p_vec: Vec3 = ray.direction().cross(vector_v0_to_v2);
    let determinant: f32 = vector_v0_to_v1.dot(p_vec);

    match backface_culling {
        Backfaces::Cull => {
            // if the determinant is negative the triangle is backfacing
            // if the determinant is close to 0, the ray misses the triangle
            // This test checks both cases
            if determinant < epsilon {
                return None;
            }
        }
        Backfaces::Include => {
            // ray and triangle are parallel if det is close to 0
            if determinant.abs() < epsilon {
                return None;
            }
        }
    }

    let determinant_inverse = 1.0 / determinant;

    let t_vec: Vec3 = *ray.origin() - triangle.v0;
    let u = t_vec.dot(p_vec) * determinant_inverse;
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q_vec = t_vec.cross(vector_v0_to_v1);
    let v = ray.direction().dot(q_vec) * determinant_inverse;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    // The distance between ray origin and intersection is t.
    let t: f32 = vector_v0_to_v2.dot(q_vec) * determinant_inverse;

    // Move along the ray direction from the origin, to find the intersection
    let point_intersection = *ray.origin() + *ray.direction() * t;
    let triangle_normal = vector_v0_to_v1.cross(vector_v0_to_v2);

    return Some(Ray3D::new(point_intersection, triangle_normal));
}

/// Geometric method of computing a ray-triangle intersection
pub fn raycast_geometric(ray: &Ray3D, triangle: &Triangle) -> Option<Ray3D> {
    // Source: https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution
    let epsilon = 0.000001;

    // compute plane's normal
    let vector_v0_to_v1: Vec3 = triangle.v1 - triangle.v0;
    let vector_v0_to_v2: Vec3 = triangle.v2 - triangle.v0;
    // no need to normalize
    let triangle_normal = vector_v0_to_v1.cross(vector_v0_to_v2); // N

    // Step 1: finding P

    // check if ray and plane are parallel ?
    let n_dot_ray_direction = triangle_normal.dot(*ray.direction());
    if n_dot_ray_direction.abs() < epsilon {
        return None;
    }

    // compute d parameter using equation 2
    let d = triangle_normal.dot(triangle.v0);

    // compute t (equation 3)
    let t = (triangle_normal.dot(*ray.origin()) + d) / n_dot_ray_direction;
    // check if the triangle is in behind the ray
    if t < 0.0 {
        return None;
    } // the triangle is behind

    // compute the intersection point using equation 1
    let point_intersection = *ray.origin() + t * *ray.direction();

    // Step 2: inside-outside test

    // edge 0
    let edge0 = triangle.v1 - triangle.v0;
    let vp0 = point_intersection - triangle.v0;
    let cross = edge0.cross(vp0);
    if triangle_normal.dot(cross) < 0.0 {
        return None;
    } // P is on the right side

    // edge 1
    let edge1 = triangle.v2 - triangle.v1;
    let vp1 = point_intersection - triangle.v1;
    let cross = edge1.cross(vp1);
    if triangle_normal.dot(cross) < 0.0 {
        return None;
    } // P is on the right side

    // edge 2
    let edge2 = triangle.v0 - triangle.v2;
    let vp2 = point_intersection - triangle.v2;
    let cross = edge2.cross(vp2);
    if triangle_normal.dot(cross) < 0.0 {
        return None;
    } // P is on the right side;

    return Some(Ray3D::new(point_intersection, triangle_normal));
}
