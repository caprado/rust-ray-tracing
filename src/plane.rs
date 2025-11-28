use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::sphere::Material;
use crate::vector3d::Vector3D;

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub point: Vector3D,
    pub normal: Vector3D,
    pub material: Material,
}

impl Plane {
    pub fn new(point: Vector3D, normal: Vector3D, material: Material) -> Plane {
        Plane {
            point,
            normal: normal.normalize(),
            material,
        }
    }
}

impl Hittable for Plane {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let denom = self.normal.dot(ray.direction);

        // Check if ray is parallel to plane
        if denom.abs() < 1e-8 {
            return None;
        }

        let t = (self.point - ray.origin).dot(self.normal) / denom;

        if t < t_min || t > t_max {
            return None;
        }

        let point = ray.at(t);

        Some(HitRecord {
            point,
            normal: self.normal,
            t,
            material: self.material,
        })
    }
}
