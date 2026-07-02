
@group(0) @binding(0) var input: texture_storage_2d<rgba32float, read>;
@group(0) @binding(1) var output: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var<storage, read> objects: array<TracerObject>;

@group(0) @binding(3) var<uniform> view: TracerUniforms;
// @group(0) @binding(3) var skybox_texture: texture_cube<f32>;
// @group(0) @binding(4) var skybox_sampler: sampler;

struct TracerUniforms {
	sky_color: vec4<f32>,
	world_from_clip: mat4x4<f32>,
	world_position: vec3<f32>,
	sun_dir: vec3<f32>,
}


@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
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
	if hit_data.inside_swr{
		color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
	}else if hit_data.distance == 100.0 {
		// color = view.sky_color;
		let dir = ray.origin - hit_data.position;
			color = vec4<f32>(hit_data.direction + 0.5, 0.5);
	}

	let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

	textureStore(output, location, color);
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
	var d = 999999999.9;

	let len = arrayLength(&objects);
	for (var i: u32 = 0; i < len; i = i + 1){
		let cur_d = sdf_sphere(objects[i], p);
		if cur_d < d{
			d = cur_d;
		 }
	}

	return d;
}

fn distance_field_swr(p: vec3<f32>) -> f32 {
	var d = 999999999.9;

	let len = arrayLength(&objects);
	for (var i: u32 = 0; i < len; i = i + 1){
		let cur_d = sdf_swr(objects[i], p);
		if cur_d < d{
			d = cur_d;
		 }
	}

	return d;
}

fn sdf_swr(object: TracerObject, point: vec3<f32>) -> f32{
	let sphere_center = object.position;
	let sphere_radius = object.sw_radius;
	return length(point - sphere_center) - sphere_radius;
}

fn sdf_sphere(object: TracerObject, point: vec3<f32>) -> f32{
	let sphere_center = object.position;
	let sphere_radius = object.scale;
	return length(point - sphere_center) - sphere_radius;
}

fn trace(ray: Ray) -> Hit {
	var total_distance: f32 = 0.0;
	let max_distance: f32 = 100.0;
	let min_hit_distance: f32 = 0.001;
	const max_steps: i32 = 10000;

	let c : f32= 300000000.0;
	var current_pos = ray.origin;
	var vel = ray.direction * c * (0.1/c);
	for (var i: i32 = 0; i < max_steps; i = i + 1) {
		current_pos = current_pos + vel;
		let distance_to_scene = distance_field(current_pos);
		let distance_to_swr = distance_field_swr(current_pos);
		let d = min(distance_to_swr, distance_to_scene);
		let t = d/c;
		let acc = calc_g(current_pos);
		vel = normalize(vel + get_vel(acc, t)) * c * t;

		// if is_inside_swr(current_pos){
		// 	return Hit(total_distance, current_pos, vec3<f32>(0.0), vec3<f32>(1.0, 0.0, 0.0), normalize(vel), i, true);
		// }

		if (d < min_hit_distance) {
			return Hit(total_distance, current_pos, vec3<f32>(0.0), vec3<f32>(1.0, 0.0, 0.0), normalize(vel), i, distance_to_swr < 0);
		}

		if (total_distance > max_distance) {
			break;
		}

		total_distance = total_distance + d;
	}

	return Hit(max_distance, vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0, 0.0, 0.0), normalize(vel), max_steps, is_inside_swr(current_pos));
}

fn is_inside_swr(p: vec3<f32>) -> bool{
	let len = arrayLength(&objects);
	for (var i: u32 = 0; i < len; i = i + 1){
		let d = length(p - objects[i].position);
		if d <= objects[i].sw_radius{
			return true;
		}
	}
	return false;
}

fn get_vel(g: vec3<f32>, t: f32) -> vec3<f32>{
	return 2.0 * g * t;
}

fn g_acc(object: TracerObject , p: vec3<f32>) -> vec3<f32>{
	let G = 6.6743e-11;
	let dir = p - object.position;
	let r = length(dir);
	let acc = (G * object.mass)/(r*r);
	return normalize(dir) * acc;
}

fn calc_g(p: vec3<f32>) -> vec3<f32>{
	let len = arrayLength(&objects);
	var acc = vec3<f32>(0);
	for (var i: u32 = 0; i < len; i = i + 1){
		acc = acc + g_acc(objects[i], p);
	}
	return acc;
}

struct Ray {
	origin: vec3<f32>,
	direction: vec3<f32>,
	energy: vec3<f32>,
}

struct Hit {
	distance: f32,
	position: vec3<f32>,
	normal: vec3<f32>,
	color: vec3<f32>,
	direction: vec3<f32>,
	steps: i32,
	inside_swr: bool
};


struct TracerObject {
	position: vec3<f32>,
	rotation: vec4<f32>,
	scale: f32,
	mass: f32,
	sw_radius: f32,
}
