struct Camera {
    position: vec3<f32>,
    _padding1: f32,
    look_at: vec3<f32>,
    _padding2: f32,
    up: vec3<f32>,
    fov: f32,
    aspect_ratio: f32,
    _padding3: f32,
    _padding4: f32,
    _padding5: f32,
}

struct Material {
    color: vec3<f32>,
    diffuse: f32,
    specular: f32,
    shininess: f32,
    reflectivity: f32,
    _padding: f32,
}

struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material: Material,
}

struct Plane {
    point: vec3<f32>,
    _padding1: f32,
    normal: vec3<f32>,
    _padding2: f32,
    material: Material,
}

struct Light {
    position: vec3<f32>,
    intensity: f32,
}

struct RenderParams {
    width: u32,
    height: u32,
    samples: u32,
    max_depth: u32,
    background_color: vec3<f32>,
    epsilon: f32,
    num_spheres: u32,
    num_planes: u32,
    num_lights: u32,
    _padding: u32,
}

@group(0) @binding(0) var<uniform> params: RenderParams;
@group(0) @binding(1) var<uniform> camera: Camera;
@group(0) @binding(2) var<storage, read> spheres: array<Sphere>;
@group(0) @binding(3) var<storage, read> planes: array<Plane>;
@group(0) @binding(4) var<storage, read> lights: array<Light>;
@group(0) @binding(5) var<storage, read_write> output: array<vec4<f32>>;

const PI: f32 = 3.14159265359;

fn pcg_hash(seed: u32) -> u32 {
    var state = seed * 747796405u + 2891336453u;
    var word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn random_f32(seed: ptr<function, u32>) -> f32 {
    *seed = pcg_hash(*seed);
    return f32(*seed) / 4294967296.0;
}

fn cast_camera_ray(camera: Camera, ndc_x: f32, ndc_y: f32) -> vec3<f32> {
    let forward = normalize(camera.look_at - camera.position);
    let right = normalize(cross(forward, camera.up));
    let up = cross(right, forward);

    let fov_adjustment = tan(camera.fov * PI / 360.0);
    let adjusted_x = ndc_x * camera.aspect_ratio * fov_adjustment;
    let adjusted_y = -ndc_y * fov_adjustment;

    return normalize(forward + right * adjusted_x + up * adjusted_y);
}

fn reflect_vec(incident: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    return incident - normal * (2.0 * dot(incident, normal));
}

fn hit_sphere(ray_origin: vec3<f32>, ray_dir: vec3<f32>, sphere: Sphere, t_min: f32, t_max: f32) -> f32 {
    let oc = ray_origin - sphere.center;
    let a = dot(ray_dir, ray_dir);
    let half_b = dot(oc, ray_dir);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = half_b * half_b - a * c;

    if (discriminant < 0.0) {
        return -1.0;
    }

    let sqrtd = sqrt(discriminant);
    var root = (-half_b - sqrtd) / a;

    if (root < t_min || root > t_max) {
        root = (-half_b + sqrtd) / a;
        if (root < t_min || root > t_max) {
            return -1.0;
        }
    }

    return root;
}

fn hit_plane(ray_origin: vec3<f32>, ray_dir: vec3<f32>, plane: Plane, t_min: f32, t_max: f32) -> f32 {
    let denom = dot(plane.normal, ray_dir);

    if (abs(denom) < 1e-8) {
        return -1.0;
    }

    let t = dot(plane.point - ray_origin, plane.normal) / denom;

    if (t < t_min || t > t_max) {
        return -1.0;
    }

    return t;
}

fn is_in_shadow(point: vec3<f32>, light_pos: vec3<f32>) -> bool {
    let direction = light_pos - point;
    let distance = length(direction);
    let dir_normalized = normalize(direction);
    let shadow_origin = point + dir_normalized * params.epsilon;

    for (var i = 0u; i < params.num_spheres; i++) {
        let t = hit_sphere(shadow_origin, dir_normalized, spheres[i], params.epsilon, distance - params.epsilon);
        if (t > 0.0) {
            return true;
        }
    }

    for (var i = 0u; i < params.num_planes; i++) {
        let t = hit_plane(shadow_origin, dir_normalized, planes[i], params.epsilon, distance - params.epsilon);
        if (t > 0.0) {
            return true;
        }
    }

    return false;
}

fn cast_ray(ray_origin: vec3<f32>, ray_dir: vec3<f32>, max_depth: u32) -> vec3<f32> {
    var color = vec3<f32>(0.0);
    var current_origin = ray_origin;
    var current_dir = ray_dir;
    var attenuation = vec3<f32>(1.0);

    for (var depth = 0u; depth < max_depth; depth++) {
        var closest_t = 1e10;
        var hit_material: Material;
        var hit_normal = vec3<f32>(0.0);
        var hit_point = vec3<f32>(0.0);
        var did_hit = false;

        for (var i = 0u; i < params.num_spheres; i++) {
            let t = hit_sphere(current_origin, current_dir, spheres[i], params.epsilon, closest_t);
            if (t > 0.0) {
                closest_t = t;
                hit_point = current_origin + current_dir * t;
                hit_normal = normalize((hit_point - spheres[i].center) / spheres[i].radius);
                hit_material = spheres[i].material;
                did_hit = true;
            }
        }

        for (var i = 0u; i < params.num_planes; i++) {
            let t = hit_plane(current_origin, current_dir, planes[i], params.epsilon, closest_t);
            if (t > 0.0) {
                closest_t = t;
                hit_point = current_origin + current_dir * t;
                hit_normal = planes[i].normal;
                hit_material = planes[i].material;
                did_hit = true;
            }
        }

        if (!did_hit) {
            color += attenuation * params.background_color;
            break;
        }

        var local_color = vec3<f32>(0.0);

        for (var i = 0u; i < params.num_lights; i++) {
            if (!is_in_shadow(hit_point, lights[i].position)) {
                let light_dir = normalize(lights[i].position - hit_point);
                let view_dir = normalize(current_origin - hit_point);

                let diffuse_strength = max(dot(light_dir, hit_normal), 0.0);
                let diffuse = hit_material.color * (hit_material.diffuse * diffuse_strength * lights[i].intensity);

                let halfway_dir = normalize(light_dir + view_dir);
                let spec_strength = pow(max(dot(halfway_dir, hit_normal), 0.0), hit_material.shininess);
                let specular = vec3<f32>(1.0) * (hit_material.specular * spec_strength * lights[i].intensity);

                local_color += diffuse + specular;
            }
        }

        color += attenuation * local_color;

        if (hit_material.reflectivity > 0.0) {
            attenuation *= hit_material.reflectivity;
            current_origin = hit_point + hit_normal * params.epsilon;
            current_dir = reflect_vec(current_dir, hit_normal);
        } else {
            break;
        }
    }

    return clamp(color, vec3<f32>(0.0), vec3<f32>(1.0));
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;

    if (x >= params.width || y >= params.height) {
        return;
    }

    var rng_seed = (y * params.width + x) * 1000u + 1u;

    var color = vec3<f32>(0.0);

    if (params.samples > 1u) {
        for (var s = 0u; s < params.samples; s++) {
            let offset_x = random_f32(&rng_seed);
            let offset_y = random_f32(&rng_seed);

            let ndc_x = ((f32(x) + offset_x) / f32(params.width)) * 2.0 - 1.0;
            let ndc_y = ((f32(y) + offset_y) / f32(params.height)) * 2.0 - 1.0;

            let ray_dir = cast_camera_ray(camera, ndc_x, ndc_y);
            color += cast_ray(camera.position, ray_dir, params.max_depth);
        }
        color /= f32(params.samples);
    } else {
        let ndc_x = ((f32(x) + 0.5) / f32(params.width)) * 2.0 - 1.0;
        let ndc_y = ((f32(y) + 0.5) / f32(params.height)) * 2.0 - 1.0;

        let ray_dir = cast_camera_ray(camera, ndc_x, ndc_y);
        color = cast_ray(camera.position, ray_dir, params.max_depth);
    }

    let pixel_index = y * params.width + x;
    output[pixel_index] = vec4<f32>(color, 1.0);
}
