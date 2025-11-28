# Rust Ray Tracer

A ray tracer built in Rust with both CPU and GPU implementations. Started as a learning project to understand graphics programming fundamentals then extended with GPU compute shaders for a massive performance boost.

![Animation](https://github.com/caprado/rust-ray-tracing/blob/master/animation.gif)

## Performance Comparison

I implemented the same algorithm twice - once for CPU using Rayon, and once for GPU using WebGPU compute shaders.

| Resolution | Samples | Frames | CPU      | GPU   | Speedup |
| ---------- | ------- | ------ | -------- | ----- | ------- |
| 800×600    | 2       | 36     | 6m 11s   | 9.5s  | 39×     |
| 1920×1080  | 16      | 36     | 2+ hours | 42.5s | 170×+   |

The GPU version renders Full HD frames with 16× anti-aliasing in about 1.2 seconds each.

## What's Implemented

**Rendering features:**

- Reflective surfaces with configurable bounce depth
- Shadow rays for accurate shadowing from multiple light sources
- Anti-aliasing through multi-sampling with random jitters
- Blinn-Phong shading model (diffuse + specular)
- Support for spheres and infinite planes

## Building

Requires Rust 1.71+

```bash
rustup update
```

Run on CPU:

```bash
cargo run --release
```

Run on GPU:

```bash
cargo run --release -- --gpu
```

## Implementation Notes

**GPU specifics:**

- 8×8 workgroup size seemed to be the sweet spot for my GPU
- Using PCG hash for RNG instead of pulling random numbers from CPU
- Reflections are done iteratively (not recursively) since WGSL doesn't have function recursion
- All geometry packed into storage buffers, uniform buffers for camera/params

**CPU optimizations:**

- Inline everything in the hot path
- Pre-compute 1/width and 1/height to avoid divisions in inner loops
- Use fastrand instead of thread_rng (way faster for simple random floats)
- Shadow ray calculations use squared distance until necessary to avoid extra sqrt calls

**Things I learned:**

- GPU alignment requirements are annoying (hence all the padding in the structs)
- The performance gap between CPU and GPU for embarrassingly parallel workloads is crazy
- WGSL reserved keywords will bite you (spent 20 minutes debugging "target" being a reserved word)
- Blinn-Phong halfway vector approach is noticeably easier than full Phong reflections

## Possible Extensions

Some things I might add:

- BVH acceleration structure (though not really needed for 4 objects)
- Texture mapping
- More primitives (triangles, meshes)
- Path tracing for global illumination
- Depth of field
- Motion blur
