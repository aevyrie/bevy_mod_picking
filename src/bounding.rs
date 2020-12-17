use super::*;
use bevy::prelude::*;
use core::panic;

/// Defines a bounding sphere with a center point coordinate and a radius
#[derive(Debug)]
pub struct BoundingSphere {
    mesh_radius: f32,
    transformed_radius: Option<f32>,
}

impl From<&mut Mesh> for BoundingSphere {
    fn from(mesh: &mut Mesh) -> Self {
        let mut mesh_radius = 0f32;
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
        let mut vert_iter = vertices.iter();

        let point_x = vert_iter.next().unwrap();
        // Find point y, the point furthest from point x
        let point_y = vert_iter.fold(point_x, |acc, x| {
            if x.distance(*point_x) > acc.distance(*point_x) {
                x
            } else {
                acc
            }
        });
        // Find point z, the point furthest from point y

        BoundingSphere {
            mesh_radius,
            transformed_radius: None,
        }
    }
}
