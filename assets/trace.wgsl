
@group(0) @binding(0) var input: texture_storage_2d<rgba32float, read>;

@group(0) @binding(1) var output: texture_storage_2d<rgba32float, write>;

@group(0) @binding(2) var<uniform> config: TracerUniforms;
@group(0) @binding(3) var skybox_texture: texture_cube<f32>;
@group(0) @binding(4) var skybox_sampler: sampler;

struct View {
	view_proj: mat4x4<f32>,
	view: mat4x4<f32>,
	projection: mat4x4<f32>,
	inv_view_proj: mat4x4<f32>,
	inv_view: mat4x4<f32>, // equiv to Unity's _CameraToWorld
	inv_projection: mat4x4<f32>, // equic to Unity's _CameraInverseProjection
};

struct TracerUniforms {
	sky_color: vec4<f32>,
	world_from_clip: mat4x4<f32>,
	world_position: vec3<f32>,
}

@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
	let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

	let color = vec4(0.0);

	textureStore(output, location, color);
}



@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
	let size = textureDimensions(output);

	let loc = vec2<f32>(f32(invocation_id.x), f32(invocation_id.y)) / vec2<f32>(size.xy);
	let ndc  = loc * 2.0f - 1.0f;

	var ray = createCameraRay(ndc);

	let hit_data = trace(ray);


    var final_color = hit_data.color;

    let step_factor = f32(hit_data.steps)/100.0;
    final_color = final_color * step_factor;


	var color = vec4<f32>(final_color, 1.0);
    if hit_data.distance == 100.0 {
    	color = config.sky_color;
    	// color = textureSampleLevel(skybox_texture, skybox_sampler, vec3<f32>(0.0, 0.0, 1.0), 0, 0u);
    }
	let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

	textureStore(output, location, color);
}

fn debug_matrix(uv: vec2<f32>) -> vec4<f32>{
	let ndc  = uv * 2.0f - 1.0f;
	var color = vec3<f32>(0.0, 0.0, 0.0);

	if uv.y < 0.5 {
		if uv.x < 0.5 {
			color = debugColor(config.world_from_clip[0][3] * 10);
		}else{
			color = debugColor(config.world_from_clip[1][3] * 10);
		}
	}else{
		if uv.x < 0.5 {
			color = debugColor(config.world_from_clip[3][2] * 10);
		}else{
			color = debugColor(config.world_from_clip[3][3] * 10);
		}
	}


	return vec4<f32>(color, 1.0);
}
fn debug(uv: vec2<f32>) -> vec4<f32>{
	let ndc  = uv * 2.0f - 1.0f;

	let near_clip = vec4<f32>(ndc, 0.0, 1.0);
	let far_clip  = vec4<f32>(ndc, 1.0, 1.0);

	// project into world space
	let near_world4 = config.world_from_clip * near_clip;
	let far_world4  = config.world_from_clip * far_clip;

	//Add epsilon to protect from divide by zero
	let inv_w_near = 1.0 / (near_world4.w + 1e-6);
	let inv_w_far = 1.0 / (far_world4.w + 1e-6);

	let near_world = near_world4.xyz * inv_w_near;
	let far_world  = far_world4.xyz * inv_w_far;

	let origin = config.world_position;

	let direction = normalize(near_world - origin);

	var ray = createRay(origin, direction);
	var color = vec3<f32>(0.0, 0.0, 0.0);
	if uv.y < 0.5 {
		if uv.x < 0.5 {
			color = debugColor(ray.direction.x);
		}else{
			color = debugColor(ray.direction.y);
		}
	}else{
		if uv.x < 0.5 {
			color = debugColor(ray.direction.z);
		}else{
			// color = debugColor(ray.direction.z);
			color = debugColor(origin.x * 0.1);
		}
	}

	return vec4<f32>(color, 1.0);
}

fn debugColor(v: f32) -> vec3<f32>{
	return vec3<f32>(v * 0.5 + 0.5);
}

struct Ray {
	origin: vec3<f32>,
	direction: vec3<f32>,
	energy: vec3<f32>,
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
	let near_world4 = config.world_from_clip * near_clip;
	let far_world4  = config.world_from_clip * far_clip;

	//Add epsilon to protect from divide by zero
	let inv_w_near = 1.0 / (near_world4.w + 1e-6);
	let inv_w_far = 1.0 / (far_world4.w + 1e-6);

	let near_world = near_world4.xyz * inv_w_near;
	let far_world  = far_world4.xyz * inv_w_far;

	let origin = config.world_position;

	let direction = normalize(near_world - origin);

	return createRay(origin, direction);
}

struct Hit {
    distance: f32,
    hit_pos: vec3<f32>,
    normal: vec3<f32>,
    color: vec3<f32>,
    direction: vec3<f32>,
    steps: i32,
};

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
