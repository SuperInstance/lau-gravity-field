use serde::{Deserialize, Serialize};

use crate::GravityVector;

/// A snapshot of the field at a point in time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldSnapshot {
    pub timestamp: u64,
    pub total_energy: f64,
    pub room_count: usize,
    pub center: GravityVector,
    pub flux: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let s = FieldSnapshot {
            timestamp: 100,
            total_energy: 42.0,
            room_count: 5,
            center: GravityVector::new(1.0, 2.0, 3.0),
            flux: 7.0,
        };
        assert_eq!(s.timestamp, 100);
        assert_eq!(s.room_count, 5);
    }

    #[test]
    fn test_snapshot_serde() {
        let s = FieldSnapshot {
            timestamp: 50,
            total_energy: 10.0,
            room_count: 3,
            center: GravityVector::zero(),
            flux: 1.5,
        };
        let json = serde_json::to_string(&s).unwrap();
        let s2: FieldSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(s, s2);
    }
}
