
@group(0) @binding(0) var input: texture_storage_2d<rgba32float, read>;

@group(0) @binding(1) var output: texture_storage_2d<rgba32float, write>;

@group(0) @binding(2) var<uniform> config: TracerUniforms;

@group(0) @binding(3) var<uniform> view: ViewUniform;

struct View {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>, // equiv to Unity's _CameraToWorld
    inv_projection: mat4x4<f32>, // equic to Unity's _CameraInverseProjection
};

struct ViewUniform {
    clip_from_world: mat4x4<f32>,
    unjittered_clip_from_world: mat4x4<f32>,
    world_from_clip: mat4x4<f32>,
    world_from_view: mat4x4<f32>,
    view_from_world: mat4x4<f32>,
    clip_from_view: mat4x4<f32>,
    view_from_clip: mat4x4<f32>,
    world_position: vec3<f32>,
    exposure: f32,
    viewport: vec4<f32>,
    frustum: array<vec4<f32>, 6>,
    color_grading: vec4<f32>, // simplified for example
    mip_bias: f32,
    frame_count: u32,
};

struct TracerUniforms {
    sky_color: vec4<f32>,
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

    var result = vec3<f32>(0.0f);

    var hit = trace(ray);
    result += ray.energy * shade(&ray, hit);

    let color = vec4(result , 1.0);
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
    hit.distance = -1.0f; // A negative number to represent infinity
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
    // let origin = (view.inv_view * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;

    // let direction_view = (view.inv_projection * vec4<f32>(uv, 0.0, 1.0)).xyz;
    // let direction = (view.inv_view * vec4<f32>(direction_view, 0.0)).xyz;

    let origin = view.world_position;
    let target_point = view.world_from_clip * vec4<f32>(ndc, 0.0f, 1.0f);
    let direction_point = target_point.xyz / target_point.w;
    let direction = normalize(direction_point - origin);

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
	var sphere = createSphere(vec3<f32>(0.0), 1.0f);
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
		(*bestHit).albedo = vec3(0.8f);
		(*bestHit).specular = vec3(0.3f);
	}
}

fn trace(ray: Ray) -> RayHit
{
	var bestHit = createRayHit();
	intersectGroundPlane(ray, &bestHit);
	intersectSphere(ray, &bestHit, 0);
	return bestHit;
}


fn shade(ray: ptr<function, Ray>, hit: RayHit) -> vec3<f32>
{
	if hit.distance > -1.0f
	{
		(*ray).origin = hit.position + hit.normal * 0.001f;
		(*ray).direction = reflect((*ray).direction, hit.normal);
		(*ray).energy *= hit.specular;

		//Shadows
		// var shadow = false;
		// Ray shadowRay = createRay(hit.position + hit.normal * 0.001f, -1 * _DirectionalLight.xyz);
		// RayHit shadowHit = Trace(shadowRay);
		// if (shadowHit.distance != 1.#INF)
		// {
		// 	return float3(0.0f, 0.0f, 0.0f);
		// }

		// return saturate(dot(hit.normal, _DirectionalLight.xyz) * -1) * _DirectionalLight.w * hit.albedo;
		return hit.albedo;
	}
	else
	{
		(*ray).energy = vec3(0.0f);
		return vec3<f32>(0.1f);
		// var theta = acos(ray.direction.y) / -PI;
		// var phi = atan2(ray.direction.x, -ray.direction.z) / -PI * .5f;
		// return _SkyboxTexture.SampleLevel(sampler_SkyboxTexture, float2(phi, theta), 0).xyz;
	}
}
