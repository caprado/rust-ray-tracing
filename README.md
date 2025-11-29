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

Run with adaptive quality (progressive rendering):

```bash
cargo run --release -- --gpu --adaptive
```

## Production Features

**Error handling & fallback:**

- GPU renderer returns `Result<GpuRenderer, GpuError>` instead of panicking
- Automatically falls back to CPU if GPU is unavailable or initialization fails
- Graceful degradation instead of crashes

**Memory profiling:**

- Tracks GPU memory allocation per frame
- Reports peak memory usage after rendering

**Adaptive quality:**

- Progressive rendering: 1→2→4→8→16 samples
- Provides quick preview before full quality
- Enable with `--adaptive` flag

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

**Design decisions:**

- **GPU alignment:** All structs padded to 16-byte boundaries for GPU memory layout requirements
- **Error handling:** Used `Result<T, E>` throughout GPU code to enable fallback instead of panic-on-failure
- **Memory tracking:** Calculate buffer sizes upfront to report memory usage before allocation
- **Progressive rendering:** Rerender at increasing quality levels rather than accumulating samples (simpler, more visual feedback)

**What I learned:**

- GPU fallback is critical since not all computers have discrete GPUs
- Memory profiling helps identify resolution limits before hitting OOM errors
- Progressive rendering gives better UX than a single long wait
- WGSL reserved keywords (spent 20 minutes figuring out "target" is a reserved word)

## Possible Extensions

Some things I might add:

- BVH acceleration structure (though not really needed for 4 objects)
- Texture mapping
- More primitives (triangles, meshes)
- Path tracing for global illumination
- Depth of field
- Motion blur
