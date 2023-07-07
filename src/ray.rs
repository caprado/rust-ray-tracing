use crate::vector3d::Vector3D;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vector3D,
    pub direction: Vector3D,
}

impl Ray {
    pub fn new(origin: Vector3D, direction: Vector3D) -> Ray {
        Ray { origin, direction }
    }

    // p(t) = origin + t * direction
    pub fn at(&self, t: f64) -> Vector3D {
        self.origin + (self.direction * t)
    }
}
