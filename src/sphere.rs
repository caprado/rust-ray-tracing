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

    pub fn hit(&self, ray: &Ray) -> Option<(Vector3D, Vector3D)> {
        let oc: Vector3D = ray.origin - self.center;
        let a: f64 = ray.direction.dot(ray.direction);
        let b: f64 = 2.0 * oc.dot(ray.direction);
        let c: f64 = oc.dot(oc) - self.radius * self.radius;
        let discriminant: f64 = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            let t: f64 = (-b - discriminant.sqrt()) / (2.0 * a);
            let point: Vector3D = ray.origin + t * ray.direction;
            let normal: Vector3D = (point - self.center).normalize();
            Some((point, normal))
        }
    }
}
