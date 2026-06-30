#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> sky_color: vec4<f32>;
@group(2) @binding(1) var<uniform> view: TracerView;
@group(2) @binding(2) var skybox_texture: texture_2d_array<f32>;
@group(2) @binding(3) var skybox_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {

	let loc = mesh.uv;
	let ndc  = loc * 2.0f - 1.0f;

	var ray = createCameraRay(ndc);

	let hit_data = trace(ray);


   var final_color = hit_data.color;

   let step_factor = f32(hit_data.steps)/100.0;
   final_color = final_color * step_factor;


   var color = vec4<f32>(final_color, 1.0);
   if hit_data.distance == 100.0 {
   	color = sky_color;
   // color = textureSample(skybox_texture, skybox_sampler, vec2<f32>(0.0, 1.0), 1);
   }
   return color;
}

fn createRay(origin: vec3<f32>, direction: vec3<f32>) -> Ray
{
	var ray: Ray;
	ray.origin = origin;
	ray.direction = direction;
	ray.energy = vec3<f32>(1.0f, 1.0f, 1.0f);
	return ray;
}

fn createCameraRay(ndc: vec2<f32>) -> Ray {
	let uv = vec2<f32>(ndc.x, -ndc.y);

	// clip points at near and far
	let near_clip = vec4<f32>(uv, 0.0, 1.0);
	let far_clip  = vec4<f32>(uv, 1.0, 1.0);

	// project into world space
	let near_world4 = view.world_from_clip * near_clip;
	let far_world4  = view.world_from_clip * far_clip;

	//Add epsilon to protect from divide by zero
	let inv_w_near = 1.0 / (near_world4.w + 1e-6);
	let inv_w_far = 1.0 / (far_world4.w + 1e-6);

	let near_world = near_world4.xyz * inv_w_near;
	let far_world  = far_world4.xyz * inv_w_far;

	let origin = view.world_position;

	let direction = normalize(near_world - origin);

	return createRay(origin, direction);
}

fn distance_field(p: vec3<f32>) -> f32 {
    // Simple sphere centered at (0, 0, 0) with radius 1.0
    let sphere_center = vec3<f32>(0.0, 0.0, 0.0);
    let sphere_radius = 1.0;

    // SDF for a sphere: length(p - center) - radius
    let d = length(p - sphere_center) - sphere_radius;
    return d;
}

fn trace(ray: Ray) -> Hit {
    var total_distance: f32 = 0.0;
    let max_distance: f32 = 100.0;
    let min_hit_distance: f32 = 0.001;
    const max_steps: i32 = 100;

    for (var i: i32 = 0; i < max_steps; i = i + 1) {
        let current_pos = ray.origin + ray.direction * total_distance;
        let distance_to_scene = distance_field(current_pos);

        // Check for a hit
        if (distance_to_scene < min_hit_distance) {
            // A hit occurred!
            return Hit(total_distance, current_pos, vec3<f32>(0.0), vec3<f32>(1.0, 0.0, 0.0), ray.direction, i); // Return red color for now
        }

        // Check if we marched too far
        if (total_distance > max_distance) {
            break;
        }

        // Advance the ray
        total_distance = total_distance + distance_to_scene;
    }

    // No hit found
    return Hit(max_distance, vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0, 0.0, 0.0), ray.direction, max_steps); // Return black for background
}

struct TracerView {
	world_from_clip: mat4x4<f32>,
	world_position: vec3<f32>,
}

struct Object {
// 0: shpere
// 1: plane
// 2: cube
	object_type: u32,
	position: vec3<f32>,
	rotation: vec4<f32>,
	scale: vec3<f32>
}

struct Ray {
	origin: vec3<f32>,
	direction: vec3<f32>,
	energy: vec3<f32>,
}

struct Hit {
    distance: f32,
    hit_pos: vec3<f32>,
    normal: vec3<f32>,
    color: vec3<f32>,
    direction: vec3<f32>,
    steps: i32,
};
