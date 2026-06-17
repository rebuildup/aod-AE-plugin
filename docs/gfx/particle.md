# Particle Systems

Particle simulation, physics basics, and rendering for visual effects.

## Particle System Basics

A particle system simulates many small objects (particles) following simple rules.

### Particle Structure

```rust
struct Particle {
    position: [f32; 3],
    velocity: [f32; 3],
    acceleration: [f32; 3],
    life: f32,       // remaining life (seconds)
    max_life: f32,   // initial life
    size: f32,
    color: [f32; 3], // RGB
    alpha: f32,
}
```

### Update Loop

```rust
fn update_particle(p: &mut Particle, dt: f32) {
    // Apply acceleration (gravity, wind, etc.)
    p.velocity[0] += p.acceleration[0] * dt;
    p.velocity[1] += p.acceleration[1] * dt;
    p.velocity[2] += p.acceleration[2] * dt;

    // Apply velocity
    p.position[0] += p.velocity[0] * dt;
    p.position[1] += p.velocity[1] * dt;
    p.position[2] += p.velocity[2] * dt;

    // Age
    p.life -= dt;
}

fn is_alive(p: &Particle) -> bool {
    p.life > 0.0
}
```

### Emitter

```rust
struct Emitter {
    position: [f32; 3],
    rate: f32,              // particles per second
    spread: f32,            // cone angle (radians)
    initial_velocity: f32,
    initial_life: (f32, f32), // (min, max) in seconds
    initial_size: (f32, f32),
}

fn emit(emitter: &Emitter, rng: &mut impl Rng) -> Particle {
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let spread = rng.gen_range(0.0..emitter.spread);
    let vz = spread.cos();
    let vx = angle.sin() * spread.sin();
    let vy = angle.cos() * spread.sin();

    Particle {
        position: emitter.position,
        velocity: [
            vx * emitter.initial_velocity,
            vy * emitter.initial_velocity,
            vz * emitter.initial_velocity,
        ],
        acceleration: [0.0, 0.0, 0.0], // set by forces
        life: rng.gen_range(emitter.initial_life.0..emitter.initial_life.1),
        max_life: emitter.initial_life.1,
        size: rng.gen_range(emitter.initial_size.0..emitter.initial_size.1),
        color: [1.0, 1.0, 1.0],
        alpha: 1.0,
    }
}
```

## Physics

### Gravity

```rust
fn apply_gravity(p: &mut Particle, gravity: f32) {
    p.acceleration[1] -= gravity; // Y-up convention
}
```

### Drag (Air Resistance)

```rust
fn apply_drag(p: &mut Particle, drag: f32) {
    let speed = (p.velocity[0] * p.velocity[0] + p.velocity[1] * p.velocity[1] + p.velocity[2] * p.velocity[2]).sqrt();
    if speed > 0.0 {
        let factor = 1.0 - drag * speed;
        p.velocity[0] *= factor.max(0.0);
        p.velocity[1] *= factor.max(0.0);
        p.velocity[2] *= factor.max(0.0);
    }
}
```

### Turbulence (Noise-based)

```rust
fn apply_turbulence(p: &mut Particle, time: f32, strength: f32) {
    let nx = perlin_3d(p.position[0], p.position[1], time);
    let ny = perlin_3d(p.position[1], p.position[2], time);
    let nz = perlin_3d(p.position[2], p.position[0], time);
    p.velocity[0] += nx * strength;
    p.velocity[1] += ny * strength;
    p.velocity[2] += nz * strength;
}
```

## Rendering

### Point Sprites (Simplest)

Render each particle as a single pixel or small circle.

```rust
fn render_particle(p: &Particle, buffer: &mut [PixelF32], w: usize, h: usize) {
    let x = p.position[0] as isize;
    let y = p.position[1] as isize;
    let life_ratio = p.life / p.max_life;
    let alpha = p.alpha * life_ratio;

    if x >= 0 && x < w as isize && y >= 0 && y < h as isize {
        let idx = y as usize * w + x as usize;
        let src = PixelF32 { red: p.color[0], green: p.color[1], blue: p.color[2], alpha };
        buffer[idx] = alpha_blend(&src, &buffer[idx]);
    }
}
```

### Soft Particles (Depth-aware)

Particles fade out when close to geometry (avoids hard intersection).

```rust
fn soft_particle_alpha(p: &Particle, depth_buffer: &[f32], w: usize, h: usize, softness: f32) -> f32 {
    let x = p.position[0] as usize;
    let y = p.position[1] as usize;
    if x >= w || y >= h { return 1.0; }
    let scene_depth = depth_buffer[y * w + x];
    let particle_depth = p.position[2];
    let diff = (scene_depth - particle_depth).abs();
    (diff / softness).min(1.0)
}
```

## Forces

```rust
fn apply_forces(particles: &mut [Particle], dt: f32, time: f32) {
    for p in particles.iter_mut() {
        if !is_alive(p) { continue; }
        apply_gravity(p, 9.81);
        apply_drag(p, 0.01);
        apply_turbulence(p, time, 0.5);
        update_particle(p, dt);
    }
}
```

## Performance

- 10K–100K particles is typical for real-time
- For AE (non-realtime), millions are feasible
- Use SoA (Structure of Arrays) layout for SIMD-friendly processing
- Parallel update: `rayon::par_iter_mut()` on particle slice
- Culling: skip particles outside camera frustum

```rust
// SoA layout for better cache performance
struct ParticleSystem {
    positions: Vec<[f32; 3]>,
    velocities: Vec<[f32; 3]>,
    lifetimes: Vec<f32>,
    count: usize,
}
```

## AE Integration

- AE particles are typically rendered to a buffer, then composited
- Use `define_effect!` macro for the effect registration
- Particle systems are frame-based: update all particles, then render the current frame
- For trails: render all alive particles at each frame, accumulate with motion blur
