use crate::ray::Ray;
use crate::sphere::Material;
use crate::vector3d::Vector3D;

#[derive(Debug, Clone, Copy)]
pub struct HitRecord {
    pub point: Vector3D,
    pub normal: Vector3D,
    pub t: f64,
    pub material: Material,
}

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}
