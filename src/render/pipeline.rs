use std::borrow::Cow;

use bevy::{
	prelude::*,
	render::{
		Render, RenderApp, RenderSet,
		camera::CameraProjection,
		extract_resource::{ExtractResource, ExtractResourcePlugin},
		render_asset::RenderAssets,
		render_graph::{RenderGraph, RenderLabel},
		render_resource::{
			BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
			ComputePipelineDescriptor, PipelineCache, ShaderStages, ShaderType, StorageTextureAccess, TextureFormat,
			UniformBuffer,
			binding_types::{texture_storage_2d, uniform_buffer},
		},
		renderer::{RenderDevice, RenderQueue},
		texture::GpuImage,
	},
};

use crate::{SHADER_ASSET_PATH, components::rt::RTCamera, render::node::TracerNode};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct TracerLabel;

#[derive(Resource, Reflect, ExtractResource, Clone)]
#[reflect(Resource)]
pub struct TracerRenderTextures(pub Handle<Image>, pub Handle<Image>);

#[derive(Resource, Clone, ExtractResource, ShaderType, Default)]
pub struct TracerUniforms {
	pub sky_color: LinearRgba,
	pub world_from_clip: Mat4,
	pub world_position: Vec3,
}

pub struct TracerPipelinePlugin;

impl Plugin for TracerPipelinePlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			ExtractResourcePlugin::<TracerRenderTextures>::default(),
			ExtractResourcePlugin::<TracerUniforms>::default(),
		));
		app.init_resource::<TracerUniforms>()
			.add_systems(Update, switch_textures);

		app.add_systems(First, update_tracer_uniforms);
		let render_app = app.sub_app_mut(RenderApp);

		// render_app.add_systems(Startup, init_pipeline);
		render_app.add_systems(Render, prepare_bind_groups.in_set(RenderSet::PrepareBindGroups));

		let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
		render_graph.add_node(TracerLabel, TracerNode::default());
		render_graph.add_node_edge(TracerLabel, bevy::render::graph::CameraDriverLabel);
	}

	fn finish(&self, app: &mut App) {
		let render_app = app.sub_app_mut(RenderApp);
		render_app.init_resource::<TracerPipeline>();
	}
}

fn switch_textures(images: Res<TracerRenderTextures>, mut sprite: Single<&mut Sprite>) {
	if sprite.image == images.0 {
		sprite.image = images.1.clone();
	} else {
		sprite.image = images.0.clone();
	}
}

#[derive(Resource)]
pub struct TracerPipeline {
	pub texture_bind_group_layout: BindGroupLayout,
	pub init_pipeline: CachedComputePipelineId,
	pub update_pipeline: CachedComputePipelineId,
}

impl FromWorld for TracerPipeline {
	fn from_world(world: &mut World) -> Self {
		let render_device = world.resource::<RenderDevice>();

		let texture_bind_group_layout = render_device.create_bind_group_layout(
			"TracerImages",
			&BindGroupLayoutEntries::sequential(
				ShaderStages::COMPUTE,
				(
					texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
					texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
					uniform_buffer::<TracerUniforms>(false),
				),
			),
		);
		let shader = world.load_asset(SHADER_ASSET_PATH);
		let pipeline_cache = world.resource::<PipelineCache>();
		let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
			layout: vec![texture_bind_group_layout.clone()],
			shader: shader.clone(),
			entry_point: Cow::from("init"),
			label: None,
			zero_initialize_workgroup_memory: false,
			push_constant_ranges: Default::default(),
			shader_defs: Default::default(),
		});

		let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
			layout: vec![texture_bind_group_layout.clone()],
			shader,
			entry_point: Cow::from("update"),
			label: None,
			zero_initialize_workgroup_memory: false,
			push_constant_ranges: Default::default(),
			shader_defs: Default::default(),
		});

		return TracerPipeline {
			texture_bind_group_layout,
			init_pipeline,
			update_pipeline,
		};
	}
}

#[allow(dead_code)] //Pending bevy update for RenderStartup schedule
fn init_pipeline(
	mut commands: Commands,
	render_device: Res<RenderDevice>,
	asset_server: Res<AssetServer>,
	pipeline_cache: Res<PipelineCache>,
) {
	let texture_bind_group_layout = render_device.create_bind_group_layout(
		"TracerImages",
		&BindGroupLayoutEntries::sequential(
			ShaderStages::COMPUTE,
			(
				texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
				texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
				uniform_buffer::<TracerUniforms>(false),
			),
		),
	);
	let shader = asset_server.load(SHADER_ASSET_PATH);
	let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		layout: vec![texture_bind_group_layout.clone()],
		shader: shader.clone(),
		entry_point: Cow::from("init"),
		label: None,
		zero_initialize_workgroup_memory: false,
		push_constant_ranges: Default::default(),
		shader_defs: Default::default(),
	});

	let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		layout: vec![texture_bind_group_layout.clone()],
		shader,
		entry_point: Cow::from("update"),
		label: None,
		zero_initialize_workgroup_memory: false,
		push_constant_ranges: Default::default(),
		shader_defs: Default::default(),
	});

	commands.insert_resource(TracerPipeline {
		texture_bind_group_layout,
		init_pipeline,
		update_pipeline,
	});
}

#[derive(Resource)]
pub struct TracerImageBindGroups(pub [BindGroup; 2]);

fn update_tracer_uniforms(
	mut tracer_uniforms: ResMut<TracerUniforms>,
	rt_camera: Single<(&GlobalTransform, &Projection, &Camera), With<RTCamera>>,
) {
	let (transform, projection, cam) = rt_camera.into_inner();
	let view = transform.compute_matrix().inverse();
	let clip_from_view = match projection {
		Projection::Perspective(perspective_projection) => perspective_projection.get_clip_from_view(),
		_ => unreachable!("This should never happen: Invalid projection type on RT Camera"),
	};
	let clip_from_world = clip_from_view * view;
	let world_from_clip = clip_from_world.inverse();
	// info!("clip_from_view = {:?}", clip_from_view);
	// info!("world_from_clip = {:?}", world_from_clip);

	tracer_uniforms.world_from_clip = world_from_clip;
	tracer_uniforms.world_position = transform.translation();
}

fn prepare_bind_groups(
	mut commands: Commands,
	pipeline: Res<TracerPipeline>,
	gpu_images: Res<RenderAssets<GpuImage>>,
	tracer_images: Res<TracerRenderTextures>,
	tracer_uniforms: Res<TracerUniforms>,
	render_device: Res<RenderDevice>,
	queue: Res<RenderQueue>,
) {
	let view_a = gpu_images.get(&tracer_images.0).unwrap();
	let view_b = gpu_images.get(&tracer_images.1).unwrap();

	// Uniform buffer is used here to demonstrate how to set up a uniform in a compute shader
	// Alternatives such as storage buffers or push constants may be more suitable for your use case
	let mut uniform_buffer = UniformBuffer::from(tracer_uniforms.into_inner());
	uniform_buffer.write_buffer(&render_device, &queue);

	let bind_group_0 = render_device.create_bind_group(
		None,
		&pipeline.texture_bind_group_layout,
		&BindGroupEntries::sequential((&view_a.texture_view, &view_b.texture_view, &uniform_buffer)),
	);
	let bind_group_1 = render_device.create_bind_group(
		None,
		&pipeline.texture_bind_group_layout,
		&BindGroupEntries::sequential((&view_b.texture_view, &view_a.texture_view, &uniform_buffer)),
	);
	commands.insert_resource(TracerImageBindGroups([bind_group_0, bind_group_1]));
}
