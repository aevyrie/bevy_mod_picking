use super::*;
use bevy::prelude::*;
use core::panic;

#[derive(Debug, Clone)]
pub enum BoundVol {
    None,
    Loading(Handle<Mesh>),
    Loaded(BoundingSphere),
}
/// Defines a bounding sphere with a center point coordinate and a radius
#[derive(Debug, Clone)]
pub struct BoundingSphere {
    origin: Vec3,
    radius: f32,
    scaled_radius: Option<f32>,
}

impl BoundingSphere {
    pub fn origin(&self) -> Vec3 {
        self.origin
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
}

pub fn build_bound_sphere(
    meshes: Res<Assets<Mesh>>,
    mut mesh_query: Query<
        &mut PickableMesh,
    >,
) {
    for mut pickable in &mut mesh_query.iter_mut() {
        let pickle = pickable.bounding_sphere.clone();
        if let BoundVol::Loading(handle) = pickle {
            match meshes.get(&handle) {
                Some(mesh) => pickable.bounding_sphere = BoundVol::Loaded(BoundingSphere::from(mesh)),
                None => {
                    warn!("Unable to generate bounding sphere, waiting for mesh to load. Handle:{:?}", &handle);
                    continue
                }
            }
        }
    }
}

impl From<&Mesh> for BoundingSphere {
    fn from(mesh: &Mesh) -> Self {
        // Grab a vector of vertex coordinates we can use to iterate through
        if mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            panic!("Non-TriangleList mesh supplied for bounding sphere generation")
        }
        let vertices: Vec<Vec3> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            None => panic!("Mesh does not contain vertex positions"),
            Some(vertex_values) => match &vertex_values {
                VertexAttributeValues::Float3(positions) => positions
                    .iter()
                    .map(|coordinates| Vec3::from(*coordinates))
                    .collect(),
                _ => panic!("Unexpected vertex types in ATTRIBUTE_POSITION"),
            },
        };
        let point_x = vertices[0];
        // Find point y, the point furthest from point x
        let point_y = vertices.iter().fold(point_x, |acc, x| {
            if x.distance(point_x) >= acc.distance(point_x) {
                *x
            } else {
                acc
            }
        });
        // Find point z, the point furthest from point y
        let point_z = vertices.iter().fold(point_y, |acc, x| {
            if x.distance(point_y) >= acc.distance(point_y) {
                *x
            } else {
                acc
            }
        });
        // Construct a bounding sphere using these two points as the poles
        let mut sphere = BoundingSphere {
            origin: point_y.lerp(point_z, 0.5),
            radius: point_y.distance(point_z) / 2.0,
            scaled_radius: None,
        };
        // Iteratively adjust sphere until it encloses all points
        loop {
            // Find the furthest point from the origin
            let point_n = vertices.iter().fold(point_x, |acc, x| {
                if x.distance(sphere.origin) >= acc.distance(sphere.origin) {
                    *x
                } else {
                    acc
                }
            });
            // If the furthest point is outside the sphere, we need to sdjust it
            let point_dist = point_n.distance(sphere.origin);
            if point_dist > sphere.radius {
                let radius_new = (sphere.radius + point_dist) / 2.0;
                let lerp_ratio = (point_dist - radius_new) / point_dist;
                sphere = BoundingSphere {
                    origin: sphere.origin.lerp(point_n, lerp_ratio),
                    radius: radius_new,
                    scaled_radius: None,
                };
            } else {
                return sphere;
            }
        }
    }
}
