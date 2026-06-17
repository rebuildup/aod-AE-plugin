# 3D Effects

Projection matrices, camera models, lighting, and shadow rendering.

## Projection

Maps 3D world coordinates to 2D screen coordinates.

### Perspective Projection

```rust
struct PerspectiveCamera {
    fov: f32,        // field of view in radians
    aspect: f32,     // width / height
    near: f32,       // near clip plane
    far: f32,        // far clip plane
}

impl PerspectiveCamera {
    fn projection_matrix(&self) -> [[f32; 4]; 4] {
        let f = 1.0 / (self.fov / 2.0).tan();
        let nf = 1.0 / (self.near - self.far);
        [
            [f / self.aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (self.far + self.near) * nf, -1.0],
            [0.0, 0.0, 2.0 * self.far * self.near * nf, 0.0],
        ]
    }
}
```

### Orthographic Projection

No perspective distortion — parallel lines remain parallel.

```rust
fn ortho_matrix(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let w = right - left;
    let h = top - bottom;
    let d = far - near;
    [
        [2.0 / w, 0.0, 0.0, 0.0],
        [0.0, 2.0 / h, 0.0, 0.0],
        [0.0, 0.0, -2.0 / d, 0.0],
        [-(right + left) / w, -(top + bottom) / h, -(far + near) / d, 1.0],
    ]
}
```

## Camera Transform

View matrix positions the camera in world space.

```rust
fn view_matrix(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let f = normalize(sub(target, eye));
    let s = normalize(cross(f, up));
    let u = cross(s, f);
    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0],
    ]
}
```

## Lighting

### Lambertian Diffuse

```rust
fn lambertian(normal: [f32; 3], light_dir: [f32; 3], intensity: f32) -> f32 {
    let d = dot(normal, light_dir).max(0.0);
    intensity * d
}
```

### Phong Specular

```rust
fn phong_specular(normal: [f32; 3], light_dir: [f32; 3], view_dir: [f32; 3], shininess: f32) -> f32 {
    let reflect = sub(
        scale(normal, 2.0 * dot(normal, light_dir)),
        light_dir,
    );
    let d = dot(reflect, view_dir).max(0.0);
    d.powf(shininess)
}
```

### Ambient + Diffuse + Specular

```rust
fn phong_shading(normal: [f32; 3], light_dir: [f32; 3], view_dir: [f32; 3],
                 ambient: f32, diffuse: f32, specular: f32, shininess: f32) -> f32 {
    ambient + lambertian(normal, light_dir, diffuse) + phong_specular(normal, light_dir, view_dir, shininess)
}
```

## Shadow Mapping

1. Render depth from light's perspective → shadow map
2. For each fragment, transform to light space
3. Compare fragment depth with shadow map depth
4. If fragment is behind shadow map → in shadow

```rust
fn is_in_shadow(fragment_pos_light: [f32; 3], shadow_map: &[f32], sm_w: usize, sm_h: usize) -> bool {
    let sx = ((fragment_pos_light[0] + 1.0) * 0.5 * sm_w as f32) as usize;
    let sy = ((fragment_pos_light[1] + 1.0) * 0.5 * sm_h as f32) as usize;
    if sx >= sm_w || sy >= sm_h { return false; }
    let map_depth = shadow_map[sy * sm_w + sx];
    fragment_pos_light[2] > map_depth + 0.001 // bias to avoid shadow acne
}
```

**Gotcha:** Shadow acne — add a small bias to the depth comparison to prevent self-shadowing artifacts.

## AE Context

AE is 2D, but 3D effects are common (shatter, particle world, etc.). The approach:
- AE handles 2D compositing; the plugin implements 3D math internally
- Render 3D scene to a 2D buffer, then return to AE for compositing
- Use `define_effect!` macro to register the effect; AE handles the 2D layer stack
