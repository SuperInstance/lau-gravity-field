use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::GravityVector;

/// The full gravity field across all rooms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityField {
    pub vectors: HashMap<String, GravityVector>,
    pub connections: Vec<(String, String, f64)>,
}

impl GravityField {
    pub fn new() -> Self {
        Self {
            vectors: HashMap::new(),
            connections: Vec::new(),
        }
    }

    pub fn get(&self, room_id: &str) -> Option<&GravityVector> {
        self.vectors.get(room_id)
    }

    pub fn set(&mut self, room_id: &str, vector: GravityVector) {
        self.vectors.insert(room_id.to_string(), vector);
    }

    pub fn connect(&mut self, room_a: &str, room_b: &str, coupling: f64) {
        // Remove existing connection between these rooms if any
        self.connections
            .retain(|(a, b, _)| !(a == room_a && b == room_b || a == room_b && b == room_a));
        self.connections
            .push((room_a.to_string(), room_b.to_string(), coupling));
    }

    pub fn disconnect(&mut self, room_a: &str, room_b: &str) {
        self.connections
            .retain(|(a, b, _)| !(a == room_a && b == room_b || a == room_b && b == room_a));
    }

    /// Evolve the field one timestep. Each room's gravity is influenced by connected neighbors:
    /// `new = current + Σ(coupling * (neighbor - current)) * dt`
    pub fn step(&mut self, dt: f64) {
        let mut deltas: HashMap<String, GravityVector> = HashMap::new();

        // Initialize deltas to zero for all rooms
        for room_id in self.vectors.keys() {
            deltas.insert(room_id.clone(), GravityVector::zero());
        }

        for (room_a, room_b, coupling) in &self.connections {
            if let (Some(va), Some(vb)) = (self.vectors.get(room_a), self.vectors.get(room_b)) {
                // Influence on room_a from room_b
                let diff_ab = vb.sub(va).scale(coupling * dt);
                if let Some(d) = deltas.get_mut(room_a) {
                    *d = d.add(&diff_ab);
                }
                // Influence on room_b from room_a
                let diff_ba = va.sub(vb).scale(coupling * dt);
                if let Some(d) = deltas.get_mut(room_b) {
                    *d = d.add(&diff_ba);
                }
            }
        }

        // Apply deltas
        for (room_id, delta) in deltas {
            if let Some(v) = self.vectors.get_mut(&room_id) {
                *v = v.add(&delta);
            }
        }
    }

    /// Total energy: Σ|v|² across all rooms
    pub fn total_energy(&self) -> f64 {
        self.vectors.values().map(|v| v.norm_sq()).sum()
    }

    /// Weighted average gravity (center of mass)
    pub fn center_of_mass(&self) -> GravityVector {
        if self.vectors.is_empty() {
            return GravityVector::zero();
        }
        let count = self.vectors.len() as f64;
        let sum = self.vectors.values().fold(GravityVector::zero(), |acc, v| acc.add(v));
        sum.scale(1.0 / count)
    }

    /// Total flow magnitude through connections
    pub fn flux(&self) -> f64 {
        self.connections
            .iter()
            .map(|(a, b, coupling)| {
                if let (Some(va), Some(vb)) = (self.vectors.get(a), self.vectors.get(b)) {
                    coupling * va.sub(vb).magnitude()
                } else {
                    0.0
                }
            })
            .sum()
    }
}

impl Default for GravityField {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_field() {
        let f = GravityField::new();
        assert!(f.vectors.is_empty());
        assert!(f.connections.is_empty());
    }

    #[test]
    fn test_set_and_get() {
        let mut f = GravityField::new();
        f.set("room1", GravityVector::new(1.0, 2.0, 3.0));
        let v = f.get("room1").unwrap();
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
        assert!(f.get("nonexistent").is_none());
    }

    #[test]
    fn test_set_overwrites() {
        let mut f = GravityField::new();
        f.set("room1", GravityVector::new(1.0, 0.0, 0.0));
        f.set("room1", GravityVector::new(2.0, 0.0, 0.0));
        assert_eq!(f.get("room1").unwrap().x, 2.0);
    }

    #[test]
    fn test_connect() {
        let mut f = GravityField::new();
        f.connect("a", "b", 0.5);
        assert_eq!(f.connections.len(), 1);
        assert_eq!(f.connections[0].2, 0.5);
    }

    #[test]
    fn test_connect_overwrites() {
        let mut f = GravityField::new();
        f.connect("a", "b", 0.5);
        f.connect("a", "b", 0.8);
        assert_eq!(f.connections.len(), 1);
        assert_eq!(f.connections[0].2, 0.8);
    }

    #[test]
    fn test_disconnect() {
        let mut f = GravityField::new();
        f.connect("a", "b", 0.5);
        f.disconnect("a", "b");
        assert!(f.connections.is_empty());
    }

    #[test]
    fn test_disconnect_reverse_order() {
        let mut f = GravityField::new();
        f.connect("a", "b", 0.5);
        f.disconnect("b", "a");
        assert!(f.connections.is_empty());
    }

    #[test]
    fn test_total_energy() {
        let mut f = GravityField::new();
        f.set("r1", GravityVector::new(3.0, 4.0, 0.0)); // |v|² = 25
        f.set("r2", GravityVector::new(0.0, 0.0, 5.0)); // |v|² = 25
        assert!((f.total_energy() - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_total_energy_empty() {
        let f = GravityField::new();
        assert_eq!(f.total_energy(), 0.0);
    }

    #[test]
    fn test_center_of_mass() {
        let mut f = GravityField::new();
        f.set("r1", GravityVector::new(2.0, 4.0, 6.0));
        f.set("r2", GravityVector::new(4.0, 6.0, 8.0));
        let c = f.center_of_mass();
        assert!((c.x - 3.0).abs() < 1e-10);
        assert!((c.y - 5.0).abs() < 1e-10);
        assert!((c.z - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_center_of_mass_empty() {
        let f = GravityField::new();
        assert_eq!(f.center_of_mass(), GravityVector::zero());
    }

    #[test]
    fn test_flux() {
        let mut f = GravityField::new();
        f.set("r1", GravityVector::new(1.0, 0.0, 0.0));
        f.set("r2", GravityVector::new(0.0, 1.0, 0.0));
        f.connect("r1", "r2", 1.0);
        // diff = (1,-1,0), mag = sqrt(2), coupling=1 → flux = sqrt(2)
        assert!((f.flux() - std::f64::consts::SQRT_2).abs() < 1e-10);
    }

    #[test]
    fn test_flux_no_connections() {
        let mut f = GravityField::new();
        f.set("r1", GravityVector::new(1.0, 0.0, 0.0));
        assert_eq!(f.flux(), 0.0);
    }

    #[test]
    fn test_step_two_rooms_equilibrate() {
        let mut f = GravityField::new();
        f.set("r1", GravityVector::new(10.0, 0.0, 0.0));
        f.set("r2", GravityVector::new(0.0, 0.0, 0.0));
        f.connect("r1", "r2", 0.1);

        f.step(1.0);

        let v1 = f.get("r1").unwrap();
        let v2 = f.get("r2").unwrap();

        // r1 should move toward r2 (decrease x), r2 should move toward r1 (increase x)
        assert!(v1.x < 10.0);
        assert!(v2.x > 0.0);
        // They should be closer together
        assert!(v1.x > v2.x); // r1 still ahead
    }

    #[test]
    fn test_step_conservation_tendency() {
        let mut f = GravityField::new();
        f.set("r1", GravityVector::new(5.0, 0.0, 0.0));
        f.set("r2", GravityVector::new(5.0, 0.0, 0.0));
        f.connect("r1", "r2", 0.5);

        let energy_before = f.total_energy();
        f.step(1.0);
        let energy_after = f.total_energy();

        // Identical rooms: no change
        assert!((energy_after - energy_before).abs() < 1e-10);
    }

    #[test]
    fn test_step_multiple_connections() {
        let mut f = GravityField::new();
        f.set("r1", GravityVector::new(10.0, 0.0, 0.0));
        f.set("r2", GravityVector::new(0.0, 0.0, 0.0));
        f.set("r3", GravityVector::new(0.0, 0.0, 0.0));
        f.connect("r1", "r2", 0.1);
        f.connect("r1", "r3", 0.1);
        f.connect("r2", "r3", 0.1);

        f.step(1.0);

        let v1 = f.get("r1").unwrap();
        // r1 should be pulled toward both r2 and r3
        assert!(v1.x < 10.0);
    }

    #[test]
    fn test_step_disconnected_rooms_unchanged() {
        let mut f = GravityField::new();
        f.set("r1", GravityVector::new(5.0, 3.0, 1.0));
        f.step(1.0);
        let v = f.get("r1").unwrap();
        assert_eq!(v.x, 5.0);
        assert_eq!(v.y, 3.0);
        assert_eq!(v.z, 1.0);
    }

    #[test]
    fn test_step_small_dt() {
        let mut f = GravityField::new();
        f.set("r1", GravityVector::new(10.0, 0.0, 0.0));
        f.set("r2", GravityVector::new(0.0, 0.0, 0.0));
        f.connect("r1", "r2", 0.5);

        f.step(0.01);
        let v1 = f.get("r1").unwrap();
        // Small dt → small change
        assert!(v1.x > 9.9);
    }

    #[test]
    fn test_step_convergence_to_equilibrium() {
        let mut f = GravityField::new();
        f.set("r1", GravityVector::new(10.0, 0.0, 0.0));
        f.set("r2", GravityVector::new(0.0, 0.0, 0.0));
        f.connect("r1", "r2", 0.3);

        // Run 100 steps — they should converge close to the average (5,0,0)
        for _ in 0..100 {
            f.step(0.1);
        }

        let v1 = f.get("r1").unwrap();
        let v2 = f.get("r2").unwrap();
        assert!((v1.x - 5.0).abs() < 0.1);
        assert!((v2.x - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_field_serde_roundtrip() {
        let mut f = GravityField::new();
        f.set("room1", GravityVector::new(1.0, 2.0, 3.0));
        f.connect("room1", "room2", 0.5);
        let json = serde_json::to_string(&f).unwrap();
        let f2: GravityField = serde_json::from_str(&json).unwrap();
        assert_eq!(f2.vectors.len(), 1);
        assert_eq!(f2.connections.len(), 1);
    }

    #[test]
    fn test_default() {
        let f = GravityField::default();
        assert!(f.vectors.is_empty());
    }

    #[test]
    fn test_100_timestep_simulation() {
        let mut field = GravityField::new();
        field.set("hub", GravityVector::new(10.0, 5.0, 0.0));
        field.set("spoke1", GravityVector::new(1.0, 0.0, 0.0));
        field.set("spoke2", GravityVector::new(0.0, 2.0, 0.0));
        field.set("spoke3", GravityVector::new(0.0, 0.0, 3.0));
        field.connect("hub", "spoke1", 0.2);
        field.connect("hub", "spoke2", 0.2);
        field.connect("hub", "spoke3", 0.2);
        field.connect("spoke1", "spoke2", 0.1);

        let mut energies = Vec::new();
        for _ in 0..100 {
            field.step(0.05);
            energies.push(field.total_energy());
        }

        // Energy should decrease as the field equilibrates (diffusion)
        assert!(energies[99] < energies[0]);

        // All rooms should have similar vectors at equilibrium
        let hub = field.get("hub").unwrap();
        let spoke1 = field.get("spoke1").unwrap();
        let diff = hub.sub(spoke1);
        assert!(diff.magnitude() < 2.0);
    }
}
