use gif::{Frame, Repeat};
use image::RgbaImage;
use std::fs::File;
use std::path::Path;

mod gpu_renderer;
mod hittable;
mod plane;
mod ray;
mod save_image;
mod scene;
mod sphere;
mod vector3d;

use crate::plane::Plane;
use crate::save_image::save_image;
use crate::scene::{Camera, Light, Scene};
use crate::sphere::{Color, Material, Sphere};
use crate::vector3d::Vector3D;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let use_gpu = args.contains(&"--gpu".to_string());
    let use_adaptive = args.contains(&"--adaptive".to_string());

    if use_gpu {
        println!("Using GPU rendering");
        pollster::block_on(main_gpu(use_adaptive));
    } else {
        println!("Using CPU rendering (use --gpu for GPU mode)");
        main_cpu();
    }
}

fn main_cpu() {
    main_cpu_with_settings(800, 600, 36, 2, "animation.gif");
}

fn main_cpu_with_settings(width: u32, height: u32, num_frames: usize, samples: u32, output_file: &str) {
    println!("Rendering {} frames at {}x{} with {} samples per pixel",
             num_frames, width, height, samples);
    println!("Using parallel rendering with rayon...");

    // Create scene with enhanced materials
    let mut scene = Scene {
        background_color: Color {
            r: 0.2,
            g: 0.3,
            b: 0.5,
        },
        objects: Vec::new(),
        lights: Vec::new(),
    };

    // Add ground plane
    let ground = Plane::new(
        Vector3D::new(0.0, -1.0, 0.0),
        Vector3D::new(0.0, 1.0, 0.0),
        Material {
            color: Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
            },
            diffuse: 0.8,
            specular: 0.1,
            shininess: 10.0,
            reflectivity: 0.1,
        },
    );
    scene.add_object(Box::new(ground));

    // Add central blue sphere (matte)
    let sphere1 = Sphere::new(
        Vector3D::new(0.0, 0.0, 5.0),
        1.0,
        Material {
            color: Color {
                r: 0.2,
                g: 0.4,
                b: 1.0,
            },
            diffuse: 0.7,
            specular: 0.3,
            shininess: 32.0,
            reflectivity: 0.1,
        },
    );
    scene.add_object(Box::new(sphere1));

    // Add red reflective sphere
    let sphere2 = Sphere::new(
        Vector3D::new(-2.5, 0.5, 4.0),
        0.8,
        Material {
            color: Color {
                r: 1.0,
                g: 0.2,
                b: 0.2,
            },
            diffuse: 0.3,
            specular: 0.9,
            shininess: 100.0,
            reflectivity: 0.6,
        },
    );
    scene.add_object(Box::new(sphere2));

    // Add green sphere
    let sphere3 = Sphere::new(
        Vector3D::new(2.5, 0.3, 4.5),
        0.7,
        Material {
            color: Color {
                r: 0.2,
                g: 1.0,
                b: 0.3,
            },
            diffuse: 0.6,
            specular: 0.5,
            shininess: 64.0,
            reflectivity: 0.2,
        },
    );
    scene.add_object(Box::new(sphere3));

    // Add small yellow sphere
    let sphere4 = Sphere::new(
        Vector3D::new(0.0, 1.5, 3.5),
        0.4,
        Material {
            color: Color {
                r: 1.0,
                g: 0.9,
                b: 0.2,
            },
            diffuse: 0.5,
            specular: 0.8,
            shininess: 128.0,
            reflectivity: 0.4,
        },
    );
    scene.add_object(Box::new(sphere4));

    // Create camera with proper FOV
    let aspect_ratio = width as f64 / height as f64;
    let camera = Camera::new(
        Vector3D::new(0.0, 1.0, 0.0),
        Vector3D::new(0.0, 0.5, 5.0),
        60.0,
        aspect_ratio,
    );

    let mut frames = Vec::new();

    for frame_index in 0..num_frames {
        println!("Rendering frame {}/{}...", frame_index + 1, num_frames);

        // Animate rotating light
        let angle = frame_index as f64 * 2.0 * std::f64::consts::PI / num_frames as f64;
        let light_x = angle.cos() * 3.0;
        let light_z = 5.0 + angle.sin() * 2.0;
        let light_y = 2.0 + angle.sin() * 0.5;

        scene.lights.clear();
        scene.lights.push(Light {
            position: Vector3D::new(light_x, light_y, light_z),
            intensity: 1.0,
        });

        // Add a secondary static light
        scene.lights.push(Light {
            position: Vector3D::new(-3.0, 4.0, 2.0),
            intensity: 0.5,
        });

        let image = scene.trace(&camera, width, height, samples);

        let mut frame_buffer = RgbaImage::new(width, height);

        for (x, y, pixel) in frame_buffer.enumerate_pixels_mut() {
            let color = &image[y as usize][x as usize];
            let rgba_color = image::Rgba([
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
                255,
            ]);
            *pixel = rgba_color;
        }

        let filename = format!("frame_{}.ppm", frame_index);
        let _ = save_image(&image, width, height, &filename);

        frames.push(frame_buffer);
    }

    let path = Path::new(output_file);
    let file = File::create(&path).unwrap();
    let mut encoder = gif::Encoder::new(file, width as u16, height as u16, &[]).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();

    for (i, frame) in frames.iter().enumerate() {
        println!("Encoding frame {}/{}...", i + 1, num_frames);

        let rgba_data = frame.as_raw();
        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);

        for chunk in rgba_data.chunks(4) {
            rgb_data.push(chunk[0]);
            rgb_data.push(chunk[1]);
            rgb_data.push(chunk[2]);
        }

        let mut gif_frame = Frame::from_rgb(width as u16, height as u16, &rgb_data);
        gif_frame.delay = 3;
        encoder.write_frame(&gif_frame).unwrap();
    }

    println!("Animation saved as {}", output_file);
}

async fn main_gpu(use_adaptive: bool) {
    use crate::gpu_renderer::GpuRenderer;

    let width = 1920;
    let height = 1080;
    let num_frames = 36;
    let samples = 16;
    let output_file = "animation_gpu.gif";

    println!("Rendering {} frames at {}x{} with {} samples per pixel{}",
             num_frames, width, height, samples,
             if use_adaptive { " (adaptive quality)" } else { "" });

    let mut renderer = match GpuRenderer::new().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("GPU initialization failed: {}", e);
            eprintln!("Falling back to CPU rendering...");
            return main_cpu_with_settings(width, height, num_frames, samples, output_file);
        }
    };

    println!("Using GPU: {}", renderer.gpu_name());

    let mut frames = Vec::new();

    for frame_index in 0..num_frames {
        if frame_index == 0 {
            let mem_info = renderer.memory_info();
            println!("GPU memory per frame: {:.1}MB", mem_info.total_allocated_mb);
        }

        println!("Rendering frame {}/{}...", frame_index + 1, num_frames);

        let angle = frame_index as f32 * 2.0 * std::f32::consts::PI / num_frames as f32;
        let light_x = angle.cos() * 3.0;
        let light_z = 5.0 + angle.sin() * 2.0;
        let light_y = 2.0 + angle.sin() * 0.5;

        let spheres_data = vec![
            (
                ([0.0, 0.0, 5.0], 1.0),
                ([0.2, 0.4, 1.0], 0.7, 0.3, 32.0, 0.1),
            ),
            (
                ([-2.5, 0.5, 4.0], 0.8),
                ([1.0, 0.2, 0.2], 0.3, 0.9, 100.0, 0.6),
            ),
            (
                ([2.5, 0.3, 4.5], 0.7),
                ([0.2, 1.0, 0.3], 0.6, 0.5, 64.0, 0.2),
            ),
            (
                ([0.0, 1.5, 3.5], 0.4),
                ([1.0, 0.9, 0.2], 0.5, 0.8, 128.0, 0.4),
            ),
        ];

        let planes_data = vec![
            (
                ([0.0, -1.0, 0.0], [0.0, 1.0, 0.0]),
                ([0.5, 0.5, 0.5], 0.8, 0.1, 10.0, 0.1),
            ),
        ];

        let lights_data = vec![
            ([light_x, light_y, light_z], 1.0),
            ([-3.0, 4.0, 2.0], 0.5),
        ];

        let background_color = [0.2, 0.3, 0.5];

        let image = if use_adaptive {
            match renderer.render_adaptive(
                width,
                height,
                samples,
                [0.0, 1.0, 0.0],
                [0.0, 0.5, 5.0],
                60.0,
                &spheres_data,
                &planes_data,
                &lights_data,
                background_color,
                &|current, target| {
                    if current < target {
                        println!("  Progressive quality: {}/{} samples", current, target);
                    }
                },
            ) {
                Ok(img) => img,
                Err(e) => {
                    eprintln!("GPU rendering failed: {}", e);
                    eprintln!("Falling back to CPU rendering...");
                    return main_cpu_with_settings(width, height, num_frames, samples, output_file);
                }
            }
        } else {
            match renderer.render(
                width,
                height,
                samples,
                [0.0, 1.0, 0.0],
                [0.0, 0.5, 5.0],
                60.0,
                &spheres_data,
                &planes_data,
                &lights_data,
                background_color,
            ) {
                Ok(img) => img,
                Err(e) => {
                    eprintln!("GPU rendering failed: {}", e);
                    eprintln!("Falling back to CPU rendering...");
                    return main_cpu_with_settings(width, height, num_frames, samples, output_file);
                }
            }
        };

        let mut frame_buffer = RgbaImage::new(width, height);
        for (x, y, pixel) in frame_buffer.enumerate_pixels_mut() {
            let color = &image[y as usize][x as usize];
            *pixel = image::Rgba([
                (color[0] * 255.0) as u8,
                (color[1] * 255.0) as u8,
                (color[2] * 255.0) as u8,
                255,
            ]);
        }

        frames.push(frame_buffer);
    }

    println!("Encoding GIF...");

    let path = Path::new(output_file);
    let file = File::create(&path).unwrap();
    let mut encoder = gif::Encoder::new(file, width as u16, height as u16, &[]).unwrap();
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();

    for (i, frame) in frames.iter().enumerate() {
        println!("Encoding frame {}/{}...", i + 1, num_frames);

        let rgba_data = frame.as_raw();
        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);

        for chunk in rgba_data.chunks(4) {
            rgb_data.push(chunk[0]);
            rgb_data.push(chunk[1]);
            rgb_data.push(chunk[2]);
        }

        let mut gif_frame = gif::Frame::from_rgb(width as u16, height as u16, &rgb_data);
        gif_frame.delay = 3;
        encoder.write_frame(&gif_frame).unwrap();
    }

    let mem_info = renderer.memory_info();
    println!("Peak GPU memory usage: {:.1}MB", mem_info.peak_allocated_mb);
    println!("Animation saved as {}", output_file);
}
