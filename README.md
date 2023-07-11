# Ray Tracing in Rust

This is a simple ray tracing project I've implemented in Rust. It generates images by simulating the behavior of light rays in a scene with objects such as a sphere. The ray tracing technique calculates the color of each pixel in the image by tracing the path of rays from the camera through the scene.

![Animation](https://github.com/caprado/rust-ray-tracing/blob/master/animation.gif)


## Features

- Basic ray tracing: Cast rays from the camera through each pixel and determine if they intersect with objects in the scene.
- Sphere representation: Implement a Sphere class to represent spheres that can be hit by rays. Determine if a given ray intersects with a sphere.
- Scene management: Create a Scene class that contains all the objects in the scene. Cast rays into the scene and determine the color of pixels based on the intersected objects.
- Lighting: Introduce light sources that affect the color of objects. Add support for different light intensities and calculate diffuse lighting.
- Animation: Create an animated sequence by rendering multiple frames with variations in the scene or camera parameters.

## Usage

1. Install Rust: Make sure you have Rust installed on your system. You can find installation instructions at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

2. Clone the repository: Clone this repository to your local machine.

3. Configure the scene: Adjust the scene configuration in the main.rs file. Define objects (only spheres currently), lights, camera position, and other scene parameters.

4. Build and run: Use the following command to build and run the project:

   ```shell
   cargo run
