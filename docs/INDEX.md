# aod-ae-utils Documentation Index

> **This is the single source of truth for the library. Read this first.**

## How to Use These Docs

1. Start with `README.md` (project root) for agent-mandated reading order
2. Use this index to navigate to the specific topic you need
3. Each doc is self-contained — read only what you need for the current task

---

## API Reference (`docs/api/`)

Usage-focused reference for every module. Function signatures, examples, gotchas.

| File | Module | Covers |
|------|--------|--------|
| [pixel.md](api/pixel.md) | `pixel` | ToPixel, PixelBlender, PixelMap, channel ops |
| [color.md](api/color.md) | `color` | sRGB/linear, HSV/HSL/OKLCH, luminance, kelvin |
| [buffer.md](api/buffer.md) | `buffer` | ImageBuffer, ROI, AE Layer conversion |
| [gpu.md](api/gpu.md) | `gpu` | GpuContext, TexturePool, compute pipelines |
| [cpu.md](api/cpu.md) | `cpu` | ParallelProcessor, SIMD, tiling |
| [ae-helpers.md](api/ae-helpers.md) | `ae_helpers` | LayerAccessor, param readers, cache |

---

## Graphics Processing Knowledge (`docs/gfx/`)

Deep knowledge for implementing visual effects. Wikipedia-level domain knowledge + Rust implementation guidance.

| File | Topic | Covers |
|------|-------|--------|
| [compositing.md](gfx/compositing.md) | Compositing | Alpha blending, Porter-Duff, mattes, premultiply |
| [blur-sharpen.md](gfx/blur-sharpen.md) | Blur & Sharpen | Gaussian, motion blur, DOF, unsharp mask |
| [distortion.md](gfx/distortion.md) | Distortion | Lens, warp, ripple, displacement maps |
| [keying.md](gfx/keying.md) | Keying | Chroma key, luma key, spill suppression |
| [3d-effects.md](gfx/3d-effects.md) | 3D Effects | Projection, camera, lighting, shadows |
| [temporal.md](gfx/temporal.md) | Temporal | Frame interpolation, temporal blur, optical flow |
| [color-grading.md](gfx/color-grading.md) | Color Grading | LUT, CDL, tone mapping, ACES |
| [noise-grain.md](gfx/noise-grain.md) | Noise & Grain | Perlin, simplex, film grain, denoise |
| [edge-morphology.md](gfx/edge-morphology.md) | Edge & Morphology | Sobel, Canny, dilate, erode, hit-miss |
| [light-effects.md](gfx/light-effects.md) | Light Effects | Lens flare, god rays, bloom, vignette |
| [particle.md](gfx/particle.md) | Particles | Particle systems, physics basics |

---

## Optimization & Low-Level Knowledge (`docs/optimization/`)

Rust-specific practical knowledge for high-performance implementation.

| File | Topic | Covers |
|------|-------|--------|
| [cpu-simd.md](optimization/cpu-simd.md) | CPU Optimization | rayon, SIMD (AVX2), cache, memory layout |
| [gpu-wgpu.md](optimization/gpu-wgpu.md) | GPU Optimization | wgpu, compute shaders, texture management |
| [ae-sdk.md](optimization/ae-sdk.md) | AE SDK | after-effects crate, PF_orld, pixel formats |
| [memory-layout.md](optimization/memory-layout.md) | Memory | AoS vs SoA, alignment, zero-copy in Rust |

---

## Reading Paths by Task

| Task | Read |
|------|------|
| Writing a new plugin from scratch | README → api/ae-helpers → optimization/ae-sdk → api/buffer → api/pixel |
| Implementing a blur effect | gfx/blur-sharpen → api/pixel → optimization/cpu-simd |
| Building a chroma keyer | gfx/keying → api/color → gfx/compositing |
| GPU-accelerating an effect | optimization/gpu-wgpu → api/gpu → api/buffer |
| Color grading tool | gfx/color-grading → api/color → gfx/compositing |
| Particle system | gfx/particle → optimization/cpu-simd → api/pixel |
| Optimizing hot loop | optimization/cpu-simd → optimization/memory-layout → api/cpu |
| Understanding After Effects internals | optimization/ae-sdk → api/ae-helpers |
