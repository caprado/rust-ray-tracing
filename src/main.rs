use gif::{Encoder, ExtensionData, Frame, Repeat};
use image::RgbaImage;
use std::borrow::Cow;
use std::fs::File;
use std::path::Path;

mod ray;
mod save_image;
mod scene;
mod sphere;
mod vector3d;

use crate::save_image::save_image;
use crate::scene::Camera;
use crate::scene::Light;
use crate::scene::Scene;
use crate::sphere::Color;
use crate::sphere::Material;
use crate::sphere::Sphere;
use crate::vector3d::Vector3D;

fn main() {
    let width: u32 = 800;
    let height: u32 = 600;
    let num_frames: i32 = 36;
    let output_file: &str = "animation.gif";

    let mut scene: Scene = Scene {
        spheres: Vec::new(),
        lights: Vec::new(),
        background_color: Color {
            r: 0.8,
            g: 0.8,
            b: 0.8,
        }, // Light gray background color
    };

    let camera: Camera = Camera {
        position: Vector3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        target: Vector3D {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        rotation_angle: 0.0,
    };

    let sphere1: Sphere = Sphere {
        center: Vector3D {
            x: 0.0,
            y: 0.0,
            z: 4.0,
        },
        radius: 1.0,
        material: Material {
            color: Color {
                r: 0.4,
                g: 0.4,
                b: 1.0,
            }, // Blue color
            diffuse: 0.2,
        },
    };

    scene.spheres.push(sphere1);

    let mut frames: Vec<RgbaImage> = Vec::new();

    for frame_index in 0..num_frames {
        // Calculate the light position or any other scene changes for the current frame
        let angle: f64 = frame_index as f64 * 2.0 * std::f64::consts::PI / num_frames as f64;
        let light_x: f64 = angle.cos() * 2.0;
        let light_y: f64 = angle.sin() * 2.0;
        let light_z: f64 = 1.0;
        let light_intensity: f64 = 10.0;
        let light: Light = Light {
            position: Vector3D {
                x: light_x,
                y: light_y,
                z: light_z,
            },
            intensity: light_intensity,
        };

        scene.lights.clear();
        scene.lights.push(light);

        let image: Vec<Vec<Color>> = scene.trace(&camera, width, height);

        let mut frame_buffer: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            RgbaImage::new(width, height);

        for (x, y, pixel) in frame_buffer.enumerate_pixels_mut() {
            let color: &Color = &image[y as usize][x as usize];
            let rgba_color: image::Rgba<u8> = image::Rgba([
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
                255,
            ]);
            *pixel = rgba_color;
        }

        let filename: String = format!("frame_{}.ppm", frame_index);
        let _ = save_image(&image, width, height, &filename);

        frames.push(frame_buffer);
    }

    // Save the frames as a GIF animation
    let path: &Path = Path::new(output_file);
    let file: File = File::create(&path).unwrap();
    let color_map: &[u8; 0] = &[];

    let mut encoder: Encoder<File> =
        Encoder::new(file, width as u16, height as u16, color_map).unwrap();

    encoder
        .write_extension(ExtensionData::Repetitions(Repeat::Infinite))
        .unwrap();

    let mut frame_data: Vec<u8> = Vec::new();

    encoder.set_repeat(Repeat::Infinite).unwrap();

    for frame in frames {
        let mut gif_frame: Frame<'_> = Frame::default();
        gif_frame.width = width as u16;
        gif_frame.height = height as u16;
        frame_data.extend_from_slice(&frame.into_raw()[..]);
        gif_frame.buffer = Cow::Borrowed(&frame_data[..]);
        encoder.write_frame(&gif_frame).unwrap();
    }

    println!("Animation saved as {}", output_file);
}
