use crate::ray::Ray;
use crate::sphere::Color;
use crate::sphere::Sphere;
use crate::vector3d::Vector3D;

#[derive(Debug, Clone, Copy)]
pub struct Light {
    pub position: Vector3D,
    pub intensity: f64,
}

pub struct Scene {
    pub background_color: Color,
    pub spheres: Vec<Sphere>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            spheres: Vec::new(),
            lights: vec![Light {
                position: Vector3D::new(0.0, 0.0, 0.0),
                intensity: 1.0,
            }],
            background_color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            },
        }
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn cast_ray(&self, ray: &Ray) -> Option<Color> {
        for sphere in &self.spheres {
            if let Some((point, normal)) = sphere.hit(ray) {
                let mut color: Color = Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                };
                for light in &self.lights {
                    let light_dir: Vector3D = (light.position - point).normalize();
                    let diffuse_light: f64 =
                        light.intensity * sphere.material.diffuse * light_dir.dot(normal).max(0.0);
                    color = color + sphere.material.color * diffuse_light;
                }
                return Some(color);
            }
        }
        None
    }

    pub fn trace(&self, camera: &Camera, width: u32, height: u32) -> Vec<Vec<Color>> {
        let mut image: Vec<Vec<Color>> = Vec::new();

        for y in 0..height {
            let mut row: Vec<Color> = Vec::new();

            for x in 0..width {
                let ndc_x: f64 = (x as f64) / (width as f64) * 2.0 - 1.0;
                let ndc_y: f64 = (y as f64) / (height as f64) * 2.0 - 1.0;

                let ray: Ray = camera.cast_ray(ndc_x, ndc_y);

                let color: Color = if let Some(c) = self.cast_ray(&ray) {
                    c
                } else {
                    Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    }
                };

                row.push(color);
            }

            image.push(row);
        }

        image
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub position: Vector3D,
    pub target: Vector3D,
    pub rotation_angle: f64,
}

impl Camera {
    pub fn new(position: Vector3D, target: Vector3D) -> Self {
        Self {
            position,
            target,
            rotation_angle: 0.0,
        }
    }

    pub fn rotate_around_sphere(&mut self, sphere_center: Vector3D, radius: f64, speed: f64) {
        self.rotation_angle += speed;
        let x: f64 = sphere_center.x + radius * self.rotation_angle.cos();
        let z: f64 = sphere_center.z + radius * self.rotation_angle.sin();
        self.position = Vector3D {
            x,
            y: self.position.y,
            z,
        };
        self.target = sphere_center;
    }

    // Cast a ray through the pixel at (x, y) in normalized device coordinates
    pub fn cast_ray(&self, x: f64, y: f64) -> Ray {
        let direction: Vector3D = Vector3D::new(x, y, -1.0).normalize();
        Ray::new(self.position, direction)
    }
}
