struct Ray3D {
    position: Vec3,
    direction: Vec3,
}

struct Triangle([Vec3; 3]);

impl Triangle {
    /// Returns a tuple of vertices, useful for variable assignment
    fn vertices(&self) -> (Vec3, Vec3, Vec3) {
        (self[0], self[1], self[2])
    }
}

impl core::ops::Deref for Triangle {
    type Target = [Vec3; 3];
    fn deref(self: &'_ Self) -> &'_ Self::Target {
        &self.0
    }
}

/// Takes a ray and triangle and computes the intersection and normal
fn ray_triangle_intersection(ray: Ray3D, triangle: Triangle) -> Option<Ray3D> {
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
    let C = edge0.cross(vp0);
    if triangle_normal.dot(C) < 0.0 {
        return None;
    } // P is on the right side

    // edge 1
    let edge1 = v2 - v1;
    let vp1 = point_intersection - v1;
    let C = edge1.cross(vp1);
    if triangle_normal.dot(C) < 0.0 {
        return None;
    } // P is on the right side

    // edge 2
    let edge2 = v0 - v2;
    let vp2 = point_intersection - v2;
    let C = edge2.cross(vp2);
    if triangle_normal.dot(C) < 0.0 {
        return None;
    } // P is on the right side;

    return Some(Ray3D {
        position: point_intersection,
        direction: triangle_normal,
    });
}