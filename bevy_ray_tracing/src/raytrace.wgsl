#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
#import bevy_render::view::View;

#import bevy_ray_tracing::types::{RTSettings, Camera, Ray, Object, Sphere, Quad, Material, HitRecord, PI, SHAPE_SPHERE, SHAPE_QUAD, hit_record, rng_state};

@group(0) @binding(0) var<storage, read_write> camera: Camera;
@group(0) @binding(1) var<storage, read_write> objects: array<Object>;
@group(0) @binding(2) var<storage, read_write> spheres: array<Sphere>;
@group(0) @binding(3) var<storage, read_write> quads: array<Quad>;
@group(0) @binding(4) var<storage, read_write> materials: array<Material>;
@group(0) @binding(5) var<uniform> settings: RTSettings;
@group(0) @binding(6) var<uniform> view: View;

// ---- Setup and Return ----
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = (in.uv - 0.5) * view.viewport.zw / view.viewport.w * vec2<f32>(1.0, -1.0);
    let uv_delta = 1.0 / view.viewport.zw;
    rng_state = u32((1.0 + in.uv.x) * view.viewport.z) * u32((1.0 + in.uv.y) * view.viewport.w);
    //rng_state = vec3<u32>(u32(in.uv.x * view.viewport.z), u32(in.uv.x * view.viewport.z) ^ u32(in.uv.y * view.viewport.w), u32(in.uv.x * view.viewport.z) + u32(in.uv.y * view.viewport.w));
    
    var color = vec3<f32>(0.0);
    for (var i = 0; i < settings.samples; i++) {
        let offset_r = rand() * 2.0 - 1.0;
        let offset = uv_delta * offset_r.xy * 0.5;
        
        let direction = (camera.forward + (uv.x + offset.x) * camera.right + (uv.y + offset.y) * camera.up);
        let ray = Ray(camera.position, direction);
        color += trace(ray, settings.bounces);
    }

    return vec4<f32>(color / f32(settings.samples), 1.0);
}

// ---- Ray Tracing ----
fn trace(d_ray: Ray, max_bounces: i32) -> vec3<f32> {
    var ray = d_ray;
    var incoming_light = vec3<f32>(0.0);
    var ray_color = vec3<f32>(1.0);
    
    for (var i = 0; i < max_bounces; i++) {
        if hit(ray) {
            let material = materials[hit_record.material_index];
             incoming_light += material.emissive.xyz * ray_color;
        
            // Scatter
            if material.emissive.x + material.emissive.y + material.emissive.z > 0.0 {
                // If the material is emissive then we can't scatter light
                break;
            }


            var attenuation = material.color.xyz;
            ray_color *= attenuation;
            if dot(ray_color, ray_color) == 0.0 {
                // The ray has no color
                break;
            }

            var refraction_ratio = material.ior;
            if hit_record.front_face {
                refraction_ratio = 1.0 / refraction_ratio;
            }

            ray.pos = hit_record.p;
            ray.dir = lambertian(hit_record.n) * material.roughness
                + lambertian(-hit_record.n) * material.diffuse_transmission
                + reflect(ray.dir, hit_record.n) * material.metallic
                + refract(ray.dir, hit_record.n, refraction_ratio) * material.specular_transmission;

            ray.dir *= 1.0 / sqrt(dot(ray.dir, ray.dir)); // Normalize
        } else {
            // incoming_light = ray_color * vec3<f32>(0.5, 0.7, 1.0);
            break;
        }
    }

    return incoming_light;
}

// ---- Scatter ----
fn lambertian(n: vec3<f32>) -> vec3<f32> {
    return n + rand_unit_sphere();
}

fn reflect(d: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    return d - 2.0*dot(d, n)*n;
}

fn refract(d: vec3<f32>, n: vec3<f32>, ior: f32) -> vec3<f32> {
    let cos_theta = min(dot(-d, n), 1.0);
    let sin_theta = sqrt(1.0 - cos_theta*cos_theta);
    if ior * sin_theta > 1.0 {
        // Total Internal Reflection
        return reflect(d, n);
    }

    // Schlick Approximation
    var r0 = (1-ior) / (1+ior);
    r0 = r0*r0;
    if r0 + (1-r0)*pow(1 - cos_theta, 5.0) > rand_f32() {
        return reflect(d, n);
    }

    // Snell's Law
    let r_out_perp = ior * (d + cos_theta*n);
    let r_out_parallel = -sqrt(abs(1.0 - dot(r_out_perp, r_out_perp))) * n;
    return r_out_perp + r_out_parallel;
}

// ---- Hit ----
// #import bevy_ray_tracing::hit::{hit_sphere, hit_quad};

fn hit(ray: Ray) -> bool {
    hit_record.t = 1000.0;
    var hit = false;
    for (var i = 0; i < i32(arrayLength(&objects)); i++) {
        let object = objects[i];
        var object_hit = false;

        let material = materials[object.material_index];
        let double_sided = material.double_sided == 1;

        switch object.shape_type {
            case SHAPE_SPHERE: {
                object_hit = hit_sphere(ray, spheres[object.shape_index], 0.001, hit_record.t, double_sided);
            }
            case SHAPE_QUAD: {
                object_hit = hit_quad(ray, quads[object.shape_index], 0.001, hit_record.t, double_sided);
            }
            default: {
                continue;
            }
        }

        if object_hit {
            hit = true;
            hit_record.material_index = object.material_index;
        }
    }

    return hit;
}

fn hit_sphere(ray: Ray, sphere: Sphere, t_min: f32, t_max: f32, double_sided: bool) -> bool {
    let oc = ray.pos - sphere.position;
    let a = dot(ray.dir, ray.dir);
    let half_b = dot(oc, ray.dir);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = half_b * half_b - a * c;
    if discriminant < 0.0 {
        return false;
    }

    let sqrt_d = sqrt(discriminant);
    var root = (-half_b - sqrt_d) / a;
    if root <= t_min || t_max <= root {
        root = (-half_b + sqrt_d) / a;
        if root <= t_min || t_max <= root {
            return false;
        }
    }

    // Set parameters
    let p = ray.pos + root * ray.dir;
    var n = (p - sphere.position) / sphere.radius;
    var front_face = dot(ray.dir, n) < 0.0;
    if !front_face {
        if !double_sided {
            return false;
        }
    
        n = -n;
    }

    let theta = acos(-n.y);
    let phi = atan2(-n.z, n.x) + PI;
    let uv = vec2<f32>(phi / (2*PI), theta / PI);

    hit_record = HitRecord(root, p, n, uv, front_face, -1);
    return true;
}

fn hit_quad(ray: Ray, quad: Quad, t_min: f32, t_max: f32, double_sided: bool) -> bool {
    // Plane
    let _q = quad.position + (quad.model * vec3<f32>(-0.5, 0.0, -0.5));
    let u = quad.model * vec3<f32>(0.0, 0.0, 1.0);
    let v = quad.model * vec3<f32>(1.0, 0.0, 0.0);
    var n = cross(u, v);
    let _d = dot(n, _q);

    // Hit Plane
    let denom = dot(n, ray.dir);
    if abs(denom) < 1e-8 {
        return false;
    }

    let t = (_d - dot(n, ray.pos)) / denom;
    if t <= t_min || t_max <= t {
        return false;
    }

    let p = ray.pos + t * ray.dir;
    let w = n / dot(n,n);
    let planar_hit_vector = p - _q;
    let alpha = dot(w, cross(planar_hit_vector, v));
    let beta = dot(w, cross(u, planar_hit_vector));
    if alpha < 0 || 1 < alpha || beta < 0 || 1 < beta {
        return false;
    }

    var record = HitRecord(t, p, n, vec2<f32>(alpha, beta), dot(ray.dir, n) < 0.0, -1);
    if !record.front_face {
        if !double_sided {
            return false;
        }

        record.n = -record.n;
    }
    
    hit_record = record;
    return true;
}

// ---- Random ----
fn rand_u32() -> u32 {
    rng_state = rng_state * 747796405u + 2891336453u;
    let word = ((rng_state >> ((rng_state >> 28u) + 4u)) ^ rng_state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn rand_f32() -> f32 {
    return abs(fract(f32(rand_u32()) / 3141.592653));
}

fn rand() -> vec3<f32> {
    let r = vec3<u32>(rand_u32(), rand_u32(), rand_u32());
    return abs(fract(vec3<f32>(r) / 3141.592653));
}

fn rand_unit_sphere() -> vec3<f32> {
    let p = rand() * 2.0 - 1.0;
    return p * (1.0 / dot(p, p));
}
