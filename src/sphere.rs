use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::vector3d::Vector3D;
use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Mul<f64> for Color {
    type Output = Self;

    #[inline]
    fn mul(self, scalar: f64) -> Self {
        Color {
            r: self.r * scalar,
            g: self.g * scalar,
            b: self.b * scalar,
        }
    }
}

impl Add for Color {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Color {
            r: self.r + other.r,
            g: self.g + other.g,
            b: self.b + other.b,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: Color,
    pub diffuse: f64,
    pub specular: f64,
    pub shininess: f64,
    pub reflectivity: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vector3D,
    pub radius: f64,
    pub material: Material,
}

impl Sphere {
    pub fn new(center: Vector3D, radius: f64, material: Material) -> Sphere {
        Sphere {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let half_b = oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;

        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        let point = ray.at(root);
        let normal = (point - self.center) * (1.0 / self.radius);

        Some(HitRecord {
            point,
            normal,
            t: root,
            material: self.material,
        })
    }
}
