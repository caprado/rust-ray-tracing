use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::sphere::Color;
use crate::vector3d::Vector3D;
use rayon::prelude::*;

const EPSILON: f64 = 0.001;
const MAX_DEPTH: i32 = 3;

#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub position: Vector3D,
    pub intensity: f64,
}

pub struct Scene {
    pub background_color: Color,
    pub objects: Vec<Box<dyn Hittable>>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn add_object(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }

    #[inline]
    fn is_in_shadow(&self, point: Vector3D, light_position: Vector3D) -> bool {
        let direction = light_position - point;
        let distance_sq = direction.magnitude_squared();
        let distance = distance_sq.sqrt();
        let inv_distance = 1.0 / distance;
        let dir_normalized = direction * inv_distance;
        let shadow_ray = Ray::new(point + dir_normalized * EPSILON, dir_normalized);

        for object in &self.objects {
            if object.hit(&shadow_ray, EPSILON, distance - EPSILON).is_some() {
                return true;
            }
        }
        false
    }

    #[inline]
    fn cast_ray(&self, ray: &Ray, depth: i32) -> Color {
        if depth <= 0 {
            return Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            };
        }

        let mut closest_hit: Option<HitRecord> = None;
        let mut closest_t = f64::INFINITY;

        for object in &self.objects {
            if let Some(hit) = object.hit(ray, EPSILON, closest_t) {
                closest_t = hit.t;
                closest_hit = Some(hit);
            }
        }

        if let Some(hit) = closest_hit {
            let mut color = Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            };

            // Calculate lighting
            for light in &self.lights {
                if !self.is_in_shadow(hit.point, light.position) {
                    let light_dir = (light.position - hit.point).normalize();
                    let view_dir = (ray.origin - hit.point).normalize();

                    // Diffuse lighting
                    let diffuse_strength = light_dir.dot(hit.normal).max(0.0);
                    let diffuse = hit.material.color * (hit.material.diffuse * diffuse_strength * light.intensity);

                    // Specular lighting (Blinn-Phong)
                    let halfway_dir = (light_dir + view_dir).normalize();
                    let spec_strength = halfway_dir.dot(hit.normal).max(0.0).powf(hit.material.shininess);
                    let specular = Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                    } * (hit.material.specular * spec_strength * light.intensity);

                    color = color + diffuse + specular;
                }
            }

            // Reflections
            if hit.material.reflectivity > 0.0 {
                let reflect_dir = reflect(ray.direction, hit.normal);
                let reflect_ray = Ray::new(hit.point + hit.normal * EPSILON, reflect_dir);
                let reflected_color = self.cast_ray(&reflect_ray, depth - 1);
                color = color + reflected_color * hit.material.reflectivity;
            }

            color
        } else {
            self.background_color
        }
    }

    pub fn trace(&self, camera: &Camera, width: u32, height: u32, samples: u32) -> Vec<Vec<Color>> {
        let inv_samples = 1.0 / samples as f64;
        let inv_width = 1.0 / width as f64;
        let inv_height = 1.0 / height as f64;

        (0..height)
            .into_par_iter()
            .map(|y| {
                let mut row = Vec::with_capacity(width as usize);
                let rng = fastrand::Rng::new();

                for x in 0..width {
                    let mut color = Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    };

                    if samples > 1 {
                        for _ in 0..samples {
                            let offset_x = rng.f64();
                            let offset_y = rng.f64();

                            let ndc_x = ((x as f64 + offset_x) * inv_width) * 2.0 - 1.0;
                            let ndc_y = ((y as f64 + offset_y) * inv_height) * 2.0 - 1.0;

                            let ray = camera.cast_ray(ndc_x, ndc_y);
                            color = color + self.cast_ray(&ray, MAX_DEPTH);
                        }
                        color = color * inv_samples;
                    } else {
                        let ndc_x = ((x as f64 + 0.5) * inv_width) * 2.0 - 1.0;
                        let ndc_y = ((y as f64 + 0.5) * inv_height) * 2.0 - 1.0;
                        let ray = camera.cast_ray(ndc_x, ndc_y);
                        color = self.cast_ray(&ray, MAX_DEPTH);
                    }

                    color.r = color.r.clamp(0.0, 1.0);
                    color.g = color.g.clamp(0.0, 1.0);
                    color.b = color.b.clamp(0.0, 1.0);

                    row.push(color);
                }

                row
            })
            .collect()
    }
}

fn reflect(incident: Vector3D, normal: Vector3D) -> Vector3D {
    incident - normal * (2.0 * incident.dot(normal))
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vector3D,
    pub target: Vector3D,
    pub up: Vector3D,
    pub fov: f64,
    pub aspect_ratio: f64,
}

impl Camera {
    pub fn new(position: Vector3D, target: Vector3D, fov: f64, aspect_ratio: f64) -> Self {
        Self {
            position,
            target,
            up: Vector3D::new(0.0, 1.0, 0.0),
            fov,
            aspect_ratio,
        }
    }

    #[inline]
    pub fn cast_ray(&self, ndc_x: f64, ndc_y: f64) -> Ray {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward);

        let fov_adjustment = (self.fov.to_radians() / 2.0).tan();
        let adjusted_x = ndc_x * self.aspect_ratio * fov_adjustment;
        let adjusted_y = -ndc_y * fov_adjustment;

        let direction = (forward + right * adjusted_x + up * adjusted_y).normalize();

        Ray::new(self.position, direction)
    }
}
