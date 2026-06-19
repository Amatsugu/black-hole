use std::borrow::Cow;

use bevy::{
	prelude::*,
	render::{
		Render, RenderApp,
		RenderSystems::PrepareBindGroups,
		extract_resource::ExtractResourcePlugin,
		render_asset::RenderAssets,
		render_graph::{RenderGraph, RenderLabel},
		render_resource::{
			BindGroupEntries, BindGroupLayoutDescriptor, BindGroupLayoutEntries, ComputePipelineDescriptor,
			PipelineCache, SamplerBindingType, ShaderStages, TextureSampleType, UniformBuffer,
			binding_types::{sampler, texture_cube, uniform_buffer},
		},
		renderer::{RenderDevice, RenderQueue},
		texture::GpuImage,
	},
};

use crate::render::{
	node::TracerNode,
	resources::{TracerImageBindGroups, TracerPipeline, TracerRenderTextures, TracerUniforms},
};

pub struct TracerPipelinePlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct TracerLabel;

impl Plugin for TracerPipelinePlugin
{
	fn build(&self, app: &mut App)
	{
		app.add_plugins((ExtractResourcePlugin::<TracerUniforms>::default(),));
		app.init_resource::<TracerUniforms>();

		let render_app = app.sub_app_mut(RenderApp);

		render_app.add_systems(Render, prepare_bind_groups.in_set(PrepareBindGroups));

		let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
		render_graph.add_node(TracerLabel, TracerNode::default());
		render_graph.add_node_edge(TracerLabel, bevy::render::graph::CameraDriverLabel);
	}
}

fn init_pipeline(
	mut commands: Commands,
	render_device: Res<RenderDevice>,
	asset_server: Res<AssetServer>,
	pipeline_cache: Res<PipelineCache>,
)
{
	let texture_bind_group_layout = render_device.create_bind_group_layout(
		"TracerImages",
		&BindGroupLayoutEntries::sequential(
			ShaderStages::COMPUTE,
			(
				uniform_buffer::<TracerUniforms>(false),
				texture_cube(TextureSampleType::Float { filterable: true }),
				sampler(SamplerBindingType::Filtering),
			),
		),
	);
	let descriptor = BindGroupLayoutDescriptor {
		entries: BindGroupLayoutEntries::sequential(
			ShaderStages::COMPUTE,
			(
				uniform_buffer::<TracerUniforms>(false),
				texture_cube(TextureSampleType::Float { filterable: true }),
				sampler(SamplerBindingType::Filtering),
			),
		)
		.to_vec(),
		label: Cow::from("Tracer Images"),
	};
	let shader = asset_server.load("assets/trace-compute.wgsl");
	let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		layout: vec![descriptor.clone()],
		shader: shader.clone(),
		entry_point: Some(Cow::from("init")),
		label: None,
		zero_initialize_workgroup_memory: false,
		push_constant_ranges: Default::default(),
		shader_defs: Default::default(),
	});

	let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		layout: vec![descriptor.clone()],
		shader,
		entry_point: Some(Cow::from("update")),
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

fn prepare_bind_groups(
	mut commands: Commands,
	pipeline: Res<TracerPipeline>,
	gpu_images: Res<RenderAssets<GpuImage>>,
	tracer_images: Res<TracerRenderTextures>,
	tracer_uniforms: Res<TracerUniforms>,
	render_device: Res<RenderDevice>,
	queue: Res<RenderQueue>,
)
{
	let view_a = gpu_images.get(&tracer_images.main).unwrap();
	let view_b = gpu_images.get(&tracer_images.secondary).unwrap();
	let skybox = gpu_images.get(&tracer_images.skybox).unwrap();

	// Uniform buffer is used here to demonstrate how to set up a uniform in a compute shader
	// Alternatives such as storage buffers or push constants may be more suitable for your use case
	let mut uniform_buffer = UniformBuffer::from(tracer_uniforms.into_inner());
	uniform_buffer.write_buffer(&render_device, &queue);

	let bind_group_0 = render_device.create_bind_group(
		None,
		&pipeline.texture_bind_group_layout,
		&BindGroupEntries::sequential((
			&view_a.texture_view,
			&view_b.texture_view,
			&uniform_buffer,
			&skybox.texture_view,
			&skybox.sampler,
		)),
	);
	let bind_group_1 = render_device.create_bind_group(
		None,
		&pipeline.texture_bind_group_layout,
		&BindGroupEntries::sequential((
			&view_b.texture_view,
			&view_a.texture_view,
			&uniform_buffer,
			&skybox.texture_view,
			&skybox.sampler,
		)),
	);
	commands.insert_resource(TracerImageBindGroups([bind_group_0, bind_group_1]));
}
