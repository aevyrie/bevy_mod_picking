use bevy::{
    prelude::*,
    render::mesh::{VertexAttribute, VertexAttributeValues},
    render::pipeline::PrimitiveTopology,
};

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub struct Ray3D {
    position: Vec3,
    direction: Vec3,
}

impl Ray3D {
    pub fn new(position: Vec3, direction: Vec3) -> Self {
        Ray3D {
            position,
            direction,
        }
    }
    pub fn position(&self) -> Vec3 {
        self.position
    }
    pub fn direction(&self) -> Vec3 {
        self.direction
    }
}

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub struct Triangle {
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
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

/// Takes a ray and triangle and computes the intersection and normal
pub fn ray_triangle_intersection(ray: Ray3D, triangle: Triangle) -> Option<Ray3D> {
    // Source: https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution
    let dir = ray.direction;
    let pos = ray.position;
    let (v0, v1, v2) = triangle.vertices();
    let epsilon = 0.000001;

    // compute plane's normal
    let v0v1: Vec3 = v1 - v0;
    let v0v2: Vec3 = v2 - v0;
    // no need to normalize
    let triangle_normal = v0v1.cross(v0v2); // N

    // Step 1: finding P

    // check if ray and plane are parallel ?
    let n_dot_ray_direction = triangle_normal.dot(dir);
    if n_dot_ray_direction.abs() < epsilon {
        return None;
    }

    // compute d parameter using equation 2
    let d = triangle_normal.dot(v0);

    // compute t (equation 3)
    let t = (triangle_normal.dot(pos) + d) / n_dot_ray_direction;
    // check if the triangle is in behind the ray
    if t < 0.0 {
        return None;
    } // the triangle is behind

    // compute the intersection point using equation 1
    let point_intersection = pos + t * dir;

    // Step 2: inside-outside test

    // edge 0
    let edge0 = v1 - v0;
    let vp0 = point_intersection - v0;
    let cross = edge0.cross(vp0);
    if triangle_normal.dot(cross) < 0.0 {
        return None;
    } // P is on the right side

    // edge 1
    let edge1 = v2 - v1;
    let vp1 = point_intersection - v1;
    let cross = edge1.cross(vp1);
    if triangle_normal.dot(cross) < 0.0 {
        return None;
    } // P is on the right side

    // edge 2
    let edge2 = v0 - v2;
    let vp2 = point_intersection - v2;
    let cross = edge2.cross(vp2);
    if triangle_normal.dot(cross) < 0.0 {
        return None;
    } // P is on the right side;

    return Some(Ray3D {
        position: point_intersection,
        direction: triangle_normal,
    });
}
