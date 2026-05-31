use serde::{Deserialize, Serialize};

use crate::GravityVector;

/// A sensor that reads the gravity field at a specific room.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GravitySensor {
    pub room_id: String,
}

impl GravitySensor {
    pub fn new(room_id: impl Into<String>) -> Self {
        Self {
            room_id: room_id.into(),
        }
    }

    pub fn read(&self, field: &GravityField) -> Option<GravityVector> {
        field.get(&self.room_id).copied()
    }

    pub fn read_normalized(&self, field: &GravityField) -> Option<GravityVector> {
        field.get(&self.room_id).map(|v| {
            let mag = v.magnitude();
            if mag < 1e-12 {
                GravityVector::zero()
            } else {
                GravityVector::new(v.x / mag, v.y / mag, v.z / mag)
            }
        })
    }

    pub fn alignment_with(&self, field: &GravityField, other_room: &str) -> Option<f64> {
        let self_mag = field.get(&self.room_id)?.magnitude();
        let other_mag = field.get(other_room)?.magnitude();
        if self_mag < 1e-12 || other_mag < 1e-12 {
            return Some(0.0);
        }
        Some(field.get(&self.room_id)?.dot(field.get(other_room)?) / (self_mag * other_mag))
    }
}

// Import GravityField for sensor methods
use crate::GravityField;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_field() -> GravityField {
        let mut f = GravityField::new();
        f.set("room1", GravityVector::new(3.0, 4.0, 0.0));
        f.set("room2", GravityVector::new(0.0, 0.0, 5.0));
        f
    }

    #[test]
    fn test_sensor_read() {
        let field = sample_field();
        let sensor = GravitySensor::new("room1");
        let v = sensor.read(&field).unwrap();
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn test_sensor_read_missing() {
        let field = sample_field();
        let sensor = GravitySensor::new("room99");
        assert!(sensor.read(&field).is_none());
    }

    #[test]
    fn test_sensor_read_normalized() {
        let field = sample_field();
        let sensor = GravitySensor::new("room1");
        let v = sensor.read_normalized(&field).unwrap();
        assert!((v.magnitude() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_sensor_read_normalized_zero() {
        let mut field = GravityField::new();
        field.set("room1", GravityVector::zero());
        let sensor = GravitySensor::new("room1");
        let v = sensor.read_normalized(&field).unwrap();
        assert_eq!(v, GravityVector::zero());
    }

    #[test]
    fn test_alignment_parallel() {
        let mut field = GravityField::new();
        field.set("room1", GravityVector::new(1.0, 0.0, 0.0));
        field.set("room2", GravityVector::new(2.0, 0.0, 0.0));
        let sensor = GravitySensor::new("room1");
        let alignment = sensor.alignment_with(&field, "room2").unwrap();
        assert!((alignment - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_alignment_perpendicular() {
        let mut field = GravityField::new();
        field.set("room1", GravityVector::new(1.0, 0.0, 0.0));
        field.set("room2", GravityVector::new(0.0, 1.0, 0.0));
        let sensor = GravitySensor::new("room1");
        let alignment = sensor.alignment_with(&field, "room2").unwrap();
        assert!(alignment.abs() < 1e-10);
    }

    #[test]
    fn test_alignment_opposite() {
        let mut field = GravityField::new();
        field.set("room1", GravityVector::new(1.0, 0.0, 0.0));
        field.set("room2", GravityVector::new(-1.0, 0.0, 0.0));
        let sensor = GravitySensor::new("room1");
        let alignment = sensor.alignment_with(&field, "room2").unwrap();
        assert!((alignment - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_alignment_zero_vector() {
        let mut field = GravityField::new();
        field.set("room1", GravityVector::zero());
        field.set("room2", GravityVector::new(1.0, 0.0, 0.0));
        let sensor = GravitySensor::new("room1");
        let alignment = sensor.alignment_with(&field, "room2").unwrap();
        assert_eq!(alignment, 0.0);
    }

    #[test]
    fn test_alignment_missing_room() {
        let field = sample_field();
        let sensor = GravitySensor::new("room1");
        assert!(sensor.alignment_with(&field, "room99").is_none());
    }

    #[test]
    fn test_sensor_serde() {
        let s = GravitySensor::new("kitchen");
        let json = serde_json::to_string(&s).unwrap();
        let s2: GravitySensor = serde_json::from_str(&json).unwrap();
        assert_eq!(s, s2);
    }
}
