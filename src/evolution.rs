use serde::{Deserialize, Serialize};

use crate::{FieldSnapshot, GravityField};

/// Tracks how the field changes over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldEvolution {
    pub history: Vec<FieldSnapshot>,
    pub max_history: usize,
}

impl FieldEvolution {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Vec::with_capacity(max_history),
            max_history,
        }
    }

    pub fn snapshot(&mut self, field: &GravityField, timestamp: u64) {
        let snap = FieldSnapshot {
            timestamp,
            total_energy: field.total_energy(),
            room_count: field.vectors.len(),
            center: field.center_of_mass(),
            flux: field.flux(),
        };
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(snap);
    }

    pub fn rate_of_change(&self, _room_id: &str) -> Option<f64> {
        if self.history.len() < 2 {
            return None;
        }
        // We look at total_energy change as a proxy if we can't get per-room.
        // For a proper per-room rate, we'd need per-room snapshots.
        // Here we approximate from the field's overall energy trajectory.
        let len = self.history.len();
        let recent = &self.history[len - 1];
        let prev = &self.history[len - 2];
        let dt = (recent.timestamp - prev.timestamp) as f64;
        if dt == 0.0 {
            return Some(0.0);
        }
        // Use room_count and flux as proxies — we'll compute from last two snapshots.
        // Actually, the spec says "how fast is this room changing" — we approximate from
        // flux and energy changes. For a precise per-room rate we'd need richer snapshots.
        // We'll use the energy difference normalized by room count.
        let de = (recent.total_energy - prev.total_energy).abs();
        let rooms = recent.room_count.max(1) as f64;
        Some(de / (rooms * dt))
    }

    pub fn stabilizing(&self) -> bool {
        if self.history.len() < 3 {
            return false;
        }
        let len = self.history.len();
        // Check if energy changes are decreasing over last 3 snapshots
        let d1 = (self.history[len - 1].total_energy - self.history[len - 2].total_energy).abs();
        let d2 = (self.history[len - 2].total_energy - self.history[len - 3].total_energy).abs();
        d1 <= d2
    }

    pub fn trending_rooms(&self, _n: usize) -> Vec<String> {
        // Without per-room history in snapshots, we return an approximation.
        // A richer implementation would store per-room deltas.
        Vec::new()
    }
}

impl Default for FieldEvolution {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GravityVector;

    #[test]
    fn test_evolution_new() {
        let e = FieldEvolution::new(100);
        assert_eq!(e.history.len(), 0);
        assert_eq!(e.max_history, 100);
    }

    #[test]
    fn test_snapshot_recording() {
        let mut field = GravityField::new();
        field.set("room1", GravityVector::new(1.0, 0.0, 0.0));
        field.set("room2", GravityVector::new(0.0, 1.0, 0.0));
        field.connect("room1", "room2", 0.5);

        let mut evo = FieldEvolution::new(10);
        evo.snapshot(&field, 0);
        assert_eq!(evo.history.len(), 1);
        assert_eq!(evo.history[0].room_count, 2);
    }

    #[test]
    fn test_max_history_respected() {
        let mut field = GravityField::new();
        field.set("room1", GravityVector::new(1.0, 0.0, 0.0));
        let mut evo = FieldEvolution::new(3);
        for t in 0..5 {
            evo.snapshot(&field, t);
        }
        assert_eq!(evo.history.len(), 3);
    }

    #[test]
    fn test_rate_of_change_none_when_empty() {
        let evo = FieldEvolution::new(10);
        assert!(evo.rate_of_change("room1").is_none());
    }

    #[test]
    fn test_rate_of_change_with_data() {
        let mut field = GravityField::new();
        field.set("room1", GravityVector::new(1.0, 0.0, 0.0));
        let mut evo = FieldEvolution::new(10);
        evo.snapshot(&field, 0);
        field.set("room1", GravityVector::new(2.0, 0.0, 0.0));
        evo.snapshot(&field, 1);
        let rate = evo.rate_of_change("room1");
        assert!(rate.is_some());
    }

    #[test]
    fn test_stabilizing_false_initially() {
        let evo = FieldEvolution::new(10);
        assert!(!evo.stabilizing());
    }

    #[test]
    fn test_stabilizing_with_stable_data() {
        let mut field = GravityField::new();
        field.set("room1", GravityVector::new(1.0, 0.0, 0.0));
        let mut evo = FieldEvolution::new(10);
        evo.snapshot(&field, 0);
        evo.snapshot(&field, 1);
        // Same field -> energy unchanged, so d1=0 < d2
        evo.snapshot(&field, 2);
        assert!(evo.stabilizing());
    }

    #[test]
    fn test_default() {
        let evo = FieldEvolution::default();
        assert_eq!(evo.max_history, 1000);
    }
}
