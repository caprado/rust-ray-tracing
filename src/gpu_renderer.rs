use bytemuck::{Pod, Zeroable};
use std::fmt;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub enum GpuError {
    NoAdapter,
    DeviceRequest(wgpu::RequestDeviceError),
    OutOfMemory { requested_mb: f64 },
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuError::NoAdapter => write!(f, "No compatible GPU adapter found"),
            GpuError::DeviceRequest(e) => write!(f, "Failed to request GPU device: {}", e),
            GpuError::OutOfMemory { requested_mb } => {
                write!(f, "Insufficient GPU memory: {:.1}MB required (try lower resolution or fewer samples)", requested_mb)
            }
        }
    }
}

impl std::error::Error for GpuError {}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuCamera {
    position: [f32; 3],
    _padding1: f32,
    look_at: [f32; 3],
    _padding2: f32,
    up: [f32; 3],
    fov: f32,
    aspect_ratio: f32,
    _padding3: f32,
    _padding4: f32,
    _padding5: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuMaterial {
    color: [f32; 3],
    diffuse: f32,
    specular: f32,
    shininess: f32,
    reflectivity: f32,
    _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuSphere {
    center: [f32; 3],
    radius: f32,
    material: GpuMaterial,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuPlane {
    point: [f32; 3],
    _padding1: f32,
    normal: [f32; 3],
    _padding2: f32,
    material: GpuMaterial,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct GpuLight {
    position: [f32; 3],
    intensity: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct RenderParams {
    width: u32,
    height: u32,
    samples: u32,
    max_depth: u32,
    background_color: [f32; 3],
    epsilon: f32,
    num_spheres: u32,
    num_planes: u32,
    num_lights: u32,
    _padding: u32,
}

pub struct GpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    memory_info: MemoryInfo,
    adapter_info: wgpu::AdapterInfo,
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_allocated_mb: f64,
    pub peak_allocated_mb: f64,
}

impl GpuRenderer {
    pub async fn new() -> Result<Self, GpuError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(GpuError::NoAdapter)?;

        let adapter_info = adapter.get_info();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("GPU Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(GpuError::DeviceRequest)?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Ray Tracer Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("raytracer.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Tracer Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Ok(Self {
            device,
            queue,
            pipeline,
            memory_info: MemoryInfo {
                total_allocated_mb: 0.0,
                peak_allocated_mb: 0.0,
            },
            adapter_info,
        })
    }

    pub fn memory_info(&self) -> &MemoryInfo {
        &self.memory_info
    }

    pub fn gpu_name(&self) -> &str {
        &self.adapter_info.name
    }

    fn calculate_memory_usage(
        &self,
        width: u32,
        height: u32,
        num_spheres: usize,
        num_planes: usize,
        num_lights: usize,
    ) -> f64 {
        let output_size = (width * height * 16) as u64;
        let staging_size = output_size;
        let params_size = std::mem::size_of::<RenderParams>() as u64;
        let camera_size = std::mem::size_of::<GpuCamera>() as u64;
        let spheres_size = (num_spheres * std::mem::size_of::<GpuSphere>()) as u64;
        let planes_size = (num_planes * std::mem::size_of::<GpuPlane>()) as u64;
        let lights_size = (num_lights * std::mem::size_of::<GpuLight>()) as u64;

        let total_bytes = output_size + staging_size + params_size + camera_size
                        + spheres_size + planes_size + lights_size;

        total_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn render(
        &mut self,
        width: u32,
        height: u32,
        samples: u32,
        camera_pos: [f32; 3],
        camera_target: [f32; 3],
        fov: f32,
        spheres_data: &[(([f32; 3], f32), ([f32; 3], f32, f32, f32, f32))],
        planes_data: &[(([f32; 3], [f32; 3]), ([f32; 3], f32, f32, f32, f32))],
        lights_data: &[([f32; 3], f32)],
        background_color: [f32; 3],
    ) -> Result<Vec<Vec<[f32; 3]>>, GpuError> {
        let memory_mb = self.calculate_memory_usage(
            width, height,
            spheres_data.len(),
            planes_data.len(),
            lights_data.len()
        );

        // Check against 2GB limit
        const MAX_MEMORY_MB: f64 = 2048.0;
        if memory_mb > MAX_MEMORY_MB {
            return Err(GpuError::OutOfMemory { requested_mb: memory_mb });
        }

        self.memory_info.total_allocated_mb = memory_mb;
        if memory_mb > self.memory_info.peak_allocated_mb {
            self.memory_info.peak_allocated_mb = memory_mb;
        }

        let aspect_ratio = width as f32 / height as f32;

        let gpu_camera = GpuCamera {
            position: camera_pos,
            _padding1: 0.0,
            look_at: camera_target,
            _padding2: 0.0,
            up: [0.0, 1.0, 0.0],
            fov,
            aspect_ratio,
            _padding3: 0.0,
            _padding4: 0.0,
            _padding5: 0.0,
        };

        let gpu_spheres: Vec<GpuSphere> = spheres_data
            .iter()
            .map(|((center, radius), (color, diffuse, specular, shininess, reflectivity))| {
                GpuSphere {
                    center: *center,
                    radius: *radius,
                    material: GpuMaterial {
                        color: *color,
                        diffuse: *diffuse,
                        specular: *specular,
                        shininess: *shininess,
                        reflectivity: *reflectivity,
                        _padding: 0.0,
                    },
                }
            })
            .collect();

        let gpu_planes: Vec<GpuPlane> = planes_data
            .iter()
            .map(|((point, normal), (color, diffuse, specular, shininess, reflectivity))| {
                GpuPlane {
                    point: *point,
                    _padding1: 0.0,
                    normal: *normal,
                    _padding2: 0.0,
                    material: GpuMaterial {
                        color: *color,
                        diffuse: *diffuse,
                        specular: *specular,
                        shininess: *shininess,
                        reflectivity: *reflectivity,
                        _padding: 0.0,
                    },
                }
            })
            .collect();

        let gpu_lights: Vec<GpuLight> = lights_data
            .iter()
            .map(|(position, intensity)| GpuLight {
                position: *position,
                intensity: *intensity,
            })
            .collect();

        let params = RenderParams {
            width,
            height,
            samples,
            max_depth: 5,
            background_color,
            epsilon: 0.001,
            num_spheres: gpu_spheres.len() as u32,
            num_planes: gpu_planes.len() as u32,
            num_lights: gpu_lights.len() as u32,
            _padding: 0,
        };

        let params_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Params Buffer"),
                contents: bytemuck::cast_slice(&[params]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let camera_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[gpu_camera]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let spheres_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Spheres Buffer"),
                contents: bytemuck::cast_slice(&gpu_spheres),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let planes_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Planes Buffer"),
                contents: bytemuck::cast_slice(&gpu_planes),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let lights_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Lights Buffer"),
                contents: bytemuck::cast_slice(&gpu_lights),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: (width * height * 16) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (width * height * 16) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: spheres_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: planes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: lights_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups((width + 7) / 8, (height + 7) / 8, 1);
        }

        encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, (width * height * 16) as u64);

        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);

        let data = buffer_slice.get_mapped_range();
        let result: &[[f32; 4]] = bytemuck::cast_slice(&data);

        let mut image = Vec::new();
        for y in 0..height {
            let mut row = Vec::new();
            for x in 0..width {
                let pixel = result[(y * width + x) as usize];
                row.push([pixel[0], pixel[1], pixel[2]]);
            }
            image.push(row);
        }

        drop(data);
        staging_buffer.unmap();

        Ok(image)
    }

    pub fn render_adaptive(
        &mut self,
        width: u32,
        height: u32,
        target_samples: u32,
        camera_pos: [f32; 3],
        camera_target: [f32; 3],
        fov: f32,
        spheres_data: &[(([f32; 3], f32), ([f32; 3], f32, f32, f32, f32))],
        planes_data: &[(([f32; 3], [f32; 3]), ([f32; 3], f32, f32, f32, f32))],
        lights_data: &[([f32; 3], f32)],
        background_color: [f32; 3],
        progress_callback: &dyn Fn(u32, u32),
    ) -> Result<Vec<Vec<[f32; 3]>>, GpuError> {
        let sample_steps = [1, 2, 4, 8, target_samples];
        let mut final_image = vec![vec![[0.0, 0.0, 0.0]; width as usize]; height as usize];

        for &samples in &sample_steps {
            if samples > target_samples {
                break;
            }

            progress_callback(samples, target_samples);

            final_image = self.render(
                width,
                height,
                samples,
                camera_pos,
                camera_target,
                fov,
                spheres_data,
                planes_data,
                lights_data,
                background_color,
            )?;
        }

        Ok(final_image)
    }
}
