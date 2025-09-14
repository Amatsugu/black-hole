
@group(0) @binding(0) var input: texture_storage_2d<rgba32float, read>;

@group(0) @binding(1) var output: texture_storage_2d<rgba32float, write>;

@group(0) @binding(2) var<uniform> config: TracerUniforms;


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

    var ray = createCameraRay2(ndc);
    var result = vec3<f32>(0.0);

    for (var i: i32 = 0; i < 8; i++){
	    var hit = trace(ray);
	    result += ray.energy * shade(&ray, hit);
		if !any(ray.energy != vec3<f32>(0.0))
		{
			break;
		}
    }

    // let clip = vec4<f32>(ndc, 0.0, 1.0);
    // let world = config.world_from_clip * clip;
    // let world_pos = world.xyz / world.w;
    // let dir = normalize(world_pos - config.world_position);

    // let color = vec4<f32>((ray.direction * 0.5) + vec3<f32>(0.5), 1.0);

    let color = vec4<f32>(result, 1.0);
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    textureStore(output, location, color);
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
    energy: vec3<f32>,
}


struct RayHit {
	distance: f32,
	position: vec3<f32>,
	normal: vec3<f32>,
	albedo: vec3<f32>,
	specular: vec3<f32>
}

struct Sphere
{
	position: vec3<f32>,
	radius: f32,
	albedo: vec3<f32>,
	specular: vec3<f32>
}

fn createRayHit() -> RayHit {
    var hit: RayHit;
    hit.position = vec3<f32>(0.0, 0.0, 0.0);
    hit.distance = 999999999999.0f;
    hit.normal = vec3<f32>(0.0, 0.0, 0.0);
    hit.albedo = vec3<f32>(0.0, 0.0, 0.0);
    hit.specular = vec3<f32>(0.0, 0.0, 0.0);
    return hit;
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

    let target_point = config.world_from_clip * vec4<f32>(ndc, 0.0, 1.0);
    let direction_point = target_point.xyz / target_point.w;
    let direction = normalize(direction_point - config.world_position);

    return createRay(config.world_position, direction);
}

fn createCameraRay2(ndc: vec2<f32>) -> Ray {
    // clip points at near and far
    let near_clip = vec4<f32>(ndc, 0.0, 1.0);
    let far_clip  = vec4<f32>(ndc, 1.0, 1.0);

    // project into world space
    let near_world4 = config.world_from_clip * near_clip;
    let far_world4  = config.world_from_clip * far_clip;

    let near_world = near_world4.xyz / near_world4.w;
    let far_world  = far_world4.xyz / far_world4.w;

    // ray starts at near plane, points toward far plane
    let origin = near_world;
    let direction = normalize(far_world - near_world);

    return createRay(origin, direction);
}

fn createSphere(position: vec3<f32>, radius: f32) -> Sphere
{
	var s: Sphere;
	s.position = position;
	s.radius = radius;
	s.albedo = vec3<f32>(0.8f, 0.8f, 0.8f);
	s.specular = vec3<f32>(0.6f, 0.6f, 0.6f);
	return s;
}

fn intersectSphere(ray: Ray, bestHit: ptr<function, RayHit>, sphereIndex: u32)
{
	//Sphere sphere = _Spheres[sphereIndex];
	var sphere = createSphere(vec3<f32>(0.0f, 0.0f, 0.0f), 1.0f);


	var d = ray.origin - sphere.position;
	var p1 = -dot(ray.direction, d);
	var p2sqr = p1 * p1 - dot(d, d) + sphere.radius * sphere.radius;
	if p2sqr < 0 {
		return;
	}
	var p2 = sqrt(p2sqr);
	// var t = p1 - p2 > 0 ? p1 - p2 : p1 + p2;
	var t = 0f;
	if p1 - p2 > 0 {
		t = p1 - p2;
	} else {
		t = p1 + p2;
	}
	if t > 0 && t < (*bestHit).distance
	{
		(*bestHit).position = ray.origin + t * ray.direction;
		(*bestHit).normal = normalize((*bestHit).position - sphere.position);
		(*bestHit).albedo = sphere.albedo;
		(*bestHit).specular = sphere.specular;
		(*bestHit).distance = t;
	}
}

fn intersectGroundPlane(ray: Ray, bestHit: ptr<function,RayHit>)
{
	var t = -ray.origin.y / ray.direction.y;
	if t > 0 && t < (*bestHit).distance
	{
		(*bestHit).distance = t;
		(*bestHit).position = ray.origin + t * ray.direction;
		(*bestHit).normal = vec3<f32>(0.0f, 1.0f, 0.0f);
		(*bestHit).albedo = vec3(0.1f);
		(*bestHit).specular = vec3(0.3f);
	}
}

fn trace(ray: Ray) -> RayHit
{
	var bestHit = createRayHit();
	intersectSphere(ray, &bestHit, 0);
	intersectGroundPlane(ray, &bestHit);
	return bestHit;
}


fn shade(ray: ptr<function, Ray>, hit: RayHit) -> vec3<f32>
{
	if hit.distance < 999999999999.0f
	{
		(*ray).origin = hit.position + hit.normal * 0.001f;
		(*ray).direction = reflect((*ray).direction, hit.normal);
		(*ray).energy *= hit.specular;

		//Shadows
		// var shadow = false;
		// var shadowRay = createRay(hit.position + hit.normal * 0.001f, -1 * _DirectionalLight.xyz);
		// var shadowHit = trace(shadowRay);
		// if (shadowHit.distance != 9999999.0f)
		// {
		// 	return float3(0.0f, 0.0f, 0.0f);
		// }

		// return saturate(dot(hit.normal, _DirectionalLight.xyz) * -1) * _DirectionalLight.w * hit.albedo;
		return hit.albedo;
	}
	else
	{
		(*ray).energy = vec3(0.0f);
		return config.sky_color.xyz;
		// var theta = acos((*ray).direction.y) / -PI;
		// var phi = atan2((*ray).direction.x, -(*ray).direction.z) / -PI * .5f;
		// return _SkyboxTexture.SampleLevel(sampler_SkyboxTexture, float2(phi, theta), 0).xyz;
	}
}
