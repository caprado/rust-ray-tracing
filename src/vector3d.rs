use std::ops::{Add, Mul, Sub};

impl Vector3D {
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Vector3D {
        Vector3D { x, y, z }
    }

    #[inline]
    pub fn dot(self, other: Vector3D) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(self, other: Vector3D) -> Vector3D {
        Vector3D::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[inline]
    pub fn magnitude(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    #[inline]
    pub fn magnitude_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline]
    pub fn normalize(self) -> Vector3D {
        let inv_mag = 1.0 / self.magnitude();
        Vector3D {
            x: self.x * inv_mag,
            y: self.y * inv_mag,
            z: self.z * inv_mag,
        }
    }
}

impl Add for Vector3D {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Vector3D {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vector3D {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Vector3D {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f64> for Vector3D {
    type Output = Self;

    #[inline]
    fn mul(self, scalar: f64) -> Self {
        Vector3D {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Mul<Vector3D> for f64 {
    type Output = Vector3D;

    #[inline]
    fn mul(self, vec: Vector3D) -> Vector3D {
        Vector3D {
            x: self * vec.x,
            y: self * vec.y,
            z: self * vec.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
