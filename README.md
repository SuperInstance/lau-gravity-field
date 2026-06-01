# lau-gravity-field

A Rust library implementing a **continuous gravity tensor field** that evolves over time. Each "room" (point in space) carries a 3D gravity vector, connected rooms influence each other through diffusion coupling, and the field evolves toward equilibrium. Includes attractors, sensors, snapshots, and evolution tracking.

Think of it as a physics engine for attention — a spatial field where gravity vectors represent attention (x), energy (y), and novelty (z), flowing and equilibrating across a connected topology.

## What This Does

- **GravityVector** — 3D vectors with arithmetic, normalization, interpolation, and serde
- **GravityField** — A map of room → vector with weighted connections; evolves via diffusion (`step(dt)`)
- **GravityAttractor** — Fixed points that pull nearby rooms toward target configurations (focus, creative, calm)
- **GravitySensor** — Reads the field at a specific room; computes normalized readings and alignment between rooms
- **FieldSnapshot** — Point-in-time recording of total energy, flux, room count, center of mass
- **FieldEvolution** — Time-series of snapshots; detects stabilization and rate of change

**65 unit tests** cover all modules.

## Key Idea

Imagine a building where every room has a "gravity" — a 3D vector encoding how much attention, energy, and novelty is present. Rooms are connected by corridors with coupling strengths. Over time, the field **diffuses**: high-gravity rooms push toward low-gravity neighbors, and the system converges to equilibrium.

The evolution rule for each timestep is:

> **v_new(room) = v(room) + Σ coupling × (v(neighbor) − v(room)) × dt**

This is exactly the **heat equation** on a graph — exponential convergence to a uniform state. The library also supports **attractors** (fixed gravitational sources that resist equilibration) for sustained patterns.

## Install

```toml
[dependencies]
lau-gravity-field = { git = "https://github.com/SuperInstance/lau-gravity-field" }
```

### Dependencies

| Crate | Purpose |
|-------|---------|
| `serde` + `serde_json` | Serialization of fields, vectors, snapshots |

## Quick Start

```rust
use lau_gravity_field::*;

// Build a field with 4 rooms
let mut field = GravityField::new();
field.set("hub", GravityVector::new(10.0, 5.0, 0.0));
field.set("spoke1", GravityVector::new(1.0, 0.0, 0.0));
field.set("spoke2", GravityVector::new(0.0, 2.0, 0.0));
field.set("spoke3", GravityVector::new(0.0, 0.0, 3.0));

// Connect rooms with coupling strengths
field.connect("hub", "spoke1", 0.2);
field.connect("hub", "spoke2", 0.2);
field.connect("hub", "spoke3", 0.2);
field.connect("spoke1", "spoke2", 0.1);

// Evolve the field
let mut evo = FieldEvolution::new(100);
for t in 0..100 {
    field.step(0.05);
    evo.snapshot(&field, t);
}

// Read the field at a room
let sensor = GravitySensor::new("hub");
let gravity = sensor.read(&field).unwrap();
println!("Hub gravity: {:?}", gravity);

// Check alignment between rooms
let alignment = sensor.alignment_with(&field, "spoke1").unwrap();
println!("Hub ↔ spoke1 alignment: {}", alignment); // cosine similarity

// Field statistics
println!("Total energy: {}", field.total_energy());
println!("Flux: {}", field.flux());
println!("Center of mass: {:?}", field.center_of_mass());
println!("Stabilizing: {}", evo.stabilizing());

// Add an attractor to maintain a creative hotspot
let attractor = GravityAttractor::new_creative(2.0, 50.0);
let pull = attractor.influence_at(&gravity);
println!("Creative pull: {:?}", pull);
```

## API Reference

### `GravityVector`

| Method | Description |
|--------|-------------|
| `new(x, y, z)` | Construct a 3D vector |
| `zero()` | Origin vector |
| `magnitude()` | Euclidean norm ‖v‖ |
| `direction()` | Unit vector [x/‖v‖, y/‖v‖, z/‖v‖] |
| `dot(other)` | Dot product ⟨v, w⟩ |
| `add(other)` / `sub(other)` | Vector addition/subtraction |
| `scale(f)` | Scalar multiplication |
| `lerp(target, t)` | Linear interpolation (clamped t ∈ [0, 1]) |
| `norm_sq()` | Squared magnitude ‖v‖² |

### `GravityField`

| Method | Description |
|--------|-------------|
| `new()` | Empty field |
| `get(room_id)` | Read gravity at a room |
| `set(room_id, vector)` | Set gravity at a room |
| `connect(a, b, coupling)` | Create/overwrite weighted connection |
| `disconnect(a, b)` | Remove connection |
| `step(dt)` | Evolve one timestep (graph diffusion) |
| `total_energy()` | Σ ‖v‖² across all rooms |
| `center_of_mass()` | Average gravity vector |
| `flux()` | Σ coupling × ‖v_a − v_b‖ across connections |

### `GravityAttractor`

| Method | Description |
|--------|-------------|
| `new(position, strength, radius)` | Custom attractor |
| `new_focus(strength, radius)` | Pulls toward high attention (+x) |
| `new_creative(strength, radius)` | Pulls toward high novelty (+z) |
| `new_calm(strength, radius)` | Pulls toward low energy (−y) |
| `influence_at(point)` | Gravitational pull (inverse-square falloff) |

### `GravitySensor`

| Method | Description |
|--------|-------------|
| `new(room_id)` | Create sensor bound to a room |
| `read(field)` | Read raw gravity vector |
| `read_normalized(field)` | Read unit-length gravity direction |
| `alignment_with(field, other_room)` | Cosine similarity between two rooms |

### `FieldSnapshot`

A `#[derive(Serialize, Deserialize)]` struct with fields: `timestamp`, `total_energy`, `room_count`, `center` (GravityVector), `flux`.

### `FieldEvolution`

| Method | Description |
|--------|-------------|
| `new(max_history)` | Tracker with bounded snapshot buffer |
| `snapshot(field, timestamp)` | Record current state |
| `rate_of_change(room_id)` | Approximate rate of change |
| `stabilizing()` | True if energy changes are decreasing |

## How It Works

### Field Evolution (Graph Diffusion)

Each `step(dt)` computes:

```
for each connection (room_a, room_b, coupling):
    delta_a += coupling * (v_b - v_a) * dt
    delta_b += coupling * (v_a - v_b) * dt

for each room:
    v(room) += delta(room)
```

This is a **forward Euler discretization** of the heat equation on the graph. The coupling acts as thermal conductivity. For stability, choose `dt` small enough that `coupling * dt < 1` for all connections.

### Convergence

For a connected field with uniform coupling, all vectors converge to the initial **center of mass** (the average of all initial gravity vectors). Total energy decreases monotonically during equilibration — the field "dissipates" differences.

### Attractors

Attractors apply an **inverse-square force** within a radius:

> F(point) = strength / ‖position − point‖² × direction

They create sustained gradients in the field, preventing full equilibration. Use them to maintain regions of high attention, creativity, or calm.

## The Math

### Graph Heat Equation

The gravity field evolves according to:

> dv_i/dt = Σ_{j~i} w_ij (v_j − v_i)

where the sum is over neighbors j of room i, and w_ij is the coupling weight. In matrix form:

> **dv/dt = −L ⊗ v**

where L is the graph Laplacian and ⊗ denotes the action on the vector-valued function. The solution converges exponentially with rate determined by the smallest non-zero eigenvalue of L.

### Energy Dissipation

Total energy E = Σ ‖v_i‖² satisfies:

> dE/dt = −2 Σ_{(i,j)∈E} w_ij ‖v_i − v_j‖² ≤ 0

Energy always decreases (or stays constant if all rooms are identical). This is the discrete analog of the Dirichlet energy.

### Inverse-Square Attractors

Attractor force follows Newton's law of gravitation:

> **F** = (G · m) / r² · **r̂**

where G·m = `strength`, r = distance, **r̂** = unit direction toward attractor. Points beyond `radius` receive no force (soft cutoff).

## License

MIT
