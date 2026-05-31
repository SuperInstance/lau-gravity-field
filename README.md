# lau-gravity-field

A continuous gravity field that flows through all rooms — a real-time tensor field where each room has a gravity vector and the field evolves over time.

## Concepts

### GravityVector
A 3D vector representing gravity at a point (room): attention (x), energy (y), novelty (z).

### GravityField
The full field across all rooms with connections between them. Evolves via diffusion: connected rooms influence each other proportional to coupling strength.

### GravityAttractor
Fixed points that pull rooms toward specific configurations (focus, creative, calm).

### FieldEvolution
Tracks how the field changes over time — snapshots, rate of change, stabilization detection.

### GravitySensor
Reads the gravity at a specific room, with normalization and alignment checks.

## Usage

```rust
use lau_gravity_field::*;

let mut field = GravityField::new();
field.set("hub", GravityVector::new(10.0, 5.0, 0.0));
field.set("spoke1", GravityVector::new(1.0, 0.0, 0.0));
field.connect("hub", "spoke1", 0.2);

// Evolve 100 timesteps
for _ in 0..100 {
    field.step(0.05);
}

// Read the field
let sensor = GravitySensor::new("hub");
let gravity = sensor.read(&field);
println!("Hub gravity: {:?}", gravity);
```

## License

MIT
