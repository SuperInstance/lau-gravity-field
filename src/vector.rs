use serde::{Deserialize, Serialize};

/// The gravity at one point (room) — a 3D vector representing attention, energy, novelty.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GravityVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl GravityVector {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn direction(&self) -> [f64; 3] {
        let mag = self.magnitude();
        if mag == 0.0 {
            [0.0, 0.0, 0.0]
        } else {
            [self.x / mag, self.y / mag, self.z / mag]
        }
    }

    pub fn dot(&self, other: &GravityVector) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn add(&self, other: &GravityVector) -> GravityVector {
        GravityVector::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(&self, other: &GravityVector) -> GravityVector {
        GravityVector::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn scale(&self, factor: f64) -> GravityVector {
        GravityVector::new(self.x * factor, self.y * factor, self.z * factor)
    }

    pub fn lerp(&self, target: &GravityVector, t: f64) -> GravityVector {
        let t = t.clamp(0.0, 1.0);
        self.add(&target.sub(self).scale(t))
    }

    pub fn norm_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}

impl Default for GravityVector {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        let v = GravityVector::zero();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
        assert_eq!(v.z, 0.0);
    }

    #[test]
    fn test_new() {
        let v = GravityVector::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_magnitude() {
        let v = GravityVector::new(3.0, 4.0, 0.0);
        assert!((v.magnitude() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_magnitude_zero() {
        assert_eq!(GravityVector::zero().magnitude(), 0.0);
    }

    #[test]
    fn test_direction() {
        let v = GravityVector::new(1.0, 0.0, 0.0);
        let d = v.direction();
        assert!((d[0] - 1.0).abs() < 1e-10);
        assert!(d[1].abs() < 1e-10);
        assert!(d[2].abs() < 1e-10);
    }

    #[test]
    fn test_direction_zero_vector() {
        let d = GravityVector::zero().direction();
        assert_eq!(d, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_dot() {
        let a = GravityVector::new(1.0, 0.0, 0.0);
        let b = GravityVector::new(0.0, 1.0, 0.0);
        assert!(a.dot(&b).abs() < 1e-10);

        let c = GravityVector::new(1.0, 0.0, 0.0);
        assert!((a.dot(&c) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_add() {
        let a = GravityVector::new(1.0, 2.0, 3.0);
        let b = GravityVector::new(4.0, 5.0, 6.0);
        let c = a.add(&b);
        assert_eq!(c.x, 5.0);
        assert_eq!(c.y, 7.0);
        assert_eq!(c.z, 9.0);
    }

    #[test]
    fn test_sub() {
        let a = GravityVector::new(4.0, 5.0, 6.0);
        let b = GravityVector::new(1.0, 2.0, 3.0);
        let c = a.sub(&b);
        assert_eq!(c.x, 3.0);
        assert_eq!(c.y, 3.0);
        assert_eq!(c.z, 3.0);
    }

    #[test]
    fn test_scale() {
        let v = GravityVector::new(1.0, 2.0, 3.0);
        let s = v.scale(2.0);
        assert_eq!(s.x, 2.0);
        assert_eq!(s.y, 4.0);
        assert_eq!(s.z, 6.0);
    }

    #[test]
    fn test_lerp() {
        let a = GravityVector::new(0.0, 0.0, 0.0);
        let b = GravityVector::new(10.0, 10.0, 10.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.x - 5.0).abs() < 1e-10);
        assert!((mid.y - 5.0).abs() < 1e-10);
        assert!((mid.z - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_lerp_clamped() {
        let a = GravityVector::new(0.0, 0.0, 0.0);
        let b = GravityVector::new(10.0, 0.0, 0.0);
        let over = a.lerp(&b, 2.0);
        assert!((over.x - 10.0).abs() < 1e-10);
        let under = a.lerp(&b, -1.0);
        assert!(under.x.abs() < 1e-10);
    }

    #[test]
    fn test_norm_sq() {
        let v = GravityVector::new(1.0, 2.0, 3.0);
        assert!((v.norm_sq() - 14.0).abs() < 1e-10);
    }

    #[test]
    fn test_default() {
        let v = GravityVector::default();
        assert_eq!(v, GravityVector::zero());
    }

    #[test]
    fn test_serde_roundtrip() {
        let v = GravityVector::new(1.5, -2.3, 0.7);
        let json = serde_json::to_string(&v).unwrap();
        let v2: GravityVector = serde_json::from_str(&json).unwrap();
        assert_eq!(v, v2);
    }
}
