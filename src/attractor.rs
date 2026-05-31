use serde::{Deserialize, Serialize};

use crate::GravityVector;

/// A fixed point that pulls rooms toward a specific gravity configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GravityAttractor {
    pub position: GravityVector,
    pub strength: f64,
    pub radius: f64,
}

impl GravityAttractor {
    pub fn new(position: GravityVector, strength: f64, radius: f64) -> Self {
        Self {
            position,
            strength,
            radius,
        }
    }

    /// Gravitational pull at a given point. Falls off with distance squared.
    pub fn influence_at(&self, point: &GravityVector) -> GravityVector {
        let diff = self.position.sub(point);
        let dist = diff.magnitude();
        if dist < 1e-12 || dist > self.radius {
            return GravityVector::zero();
        }
        let force = self.strength / (dist * dist);
        diff.scale(force)
    }

    /// Attractor that pulls toward high attention (x axis).
    pub fn new_focus(strength: f64, radius: f64) -> Self {
        Self::new(GravityVector::new(1.0, 0.0, 0.0), strength, radius)
    }

    /// Attractor that pulls toward high novelty (z axis).
    pub fn new_creative(strength: f64, radius: f64) -> Self {
        Self::new(GravityVector::new(0.0, 0.0, 1.0), strength, radius)
    }

    /// Attractor that pulls toward low energy (negative y axis).
    pub fn new_calm(strength: f64, radius: f64) -> Self {
        Self::new(GravityVector::new(0.0, -1.0, 0.0), strength, radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attractor_influence_at_position() {
        let a = GravityAttractor::new(GravityVector::new(10.0, 0.0, 0.0), 1.0, 100.0);
        let at_pos = a.influence_at(&GravityVector::new(10.0, 0.0, 0.0));
        assert!(at_pos.magnitude() < 1e-10);
    }

    #[test]
    fn test_attractor_influence_direction() {
        let a = GravityAttractor::new(GravityVector::new(10.0, 0.0, 0.0), 1.0, 100.0);
        let influence = a.influence_at(&GravityVector::new(0.0, 0.0, 0.0));
        // Should point toward attractor (positive x)
        assert!(influence.x > 0.0);
    }

    #[test]
    fn test_attractor_influence_beyond_radius() {
        let a = GravityAttractor::new(GravityVector::new(100.0, 0.0, 0.0), 1.0, 5.0);
        let influence = a.influence_at(&GravityVector::new(0.0, 0.0, 0.0));
        assert!(influence.magnitude() < 1e-10);
    }

    #[test]
    fn test_attractor_inverse_square() {
        let a = GravityAttractor::new(GravityVector::new(1.0, 0.0, 0.0), 4.0, 100.0);
        let close = a.influence_at(&GravityVector::new(0.5, 0.0, 0.0));
        let far = a.influence_at(&GravityVector::new(0.0, 0.0, 0.0));
        assert!(close.magnitude() > far.magnitude());
    }

    #[test]
    fn test_new_focus() {
        let f = GravityAttractor::new_focus(1.0, 10.0);
        assert!(f.position.x > 0.0);
        assert_eq!(f.position.y, 0.0);
        assert_eq!(f.position.z, 0.0);
    }

    #[test]
    fn test_new_creative() {
        let c = GravityAttractor::new_creative(1.0, 10.0);
        assert_eq!(c.position.x, 0.0);
        assert_eq!(c.position.y, 0.0);
        assert!(c.position.z > 0.0);
    }

    #[test]
    fn test_new_calm() {
        let c = GravityAttractor::new_calm(1.0, 10.0);
        assert_eq!(c.position.x, 0.0);
        assert!(c.position.y < 0.0);
        assert_eq!(c.position.z, 0.0);
    }

    #[test]
    fn test_serde_roundtrip() {
        let a = GravityAttractor::new_creative(2.5, 50.0);
        let json = serde_json::to_string(&a).unwrap();
        let a2: GravityAttractor = serde_json::from_str(&json).unwrap();
        assert_eq!(a, a2);
    }
}
