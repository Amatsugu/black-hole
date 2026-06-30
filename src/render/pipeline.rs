use std::borrow::Cow;

use bevy::{
	core_pipeline::schedule::camera_driver,
	prelude::*,
	render::{
		Render, RenderApp, RenderStartup, RenderSystems,
		extract_resource::ExtractResourcePlugin,
		render_asset::RenderAssets,
		render_resource::{
			binding_types::{storage_buffer_read_only, texture_storage_2d, uniform_buffer},
			*,
		},
		renderer::{RenderContext, RenderDevice, RenderGraph, RenderQueue},
		texture::GpuImage,
	},
	shader::ShaderCacheError,
};

use crate::{
	SHADER_ASSET_PATH, SIZE, WORKGROUP_SIZE,
	render::resources::{
		TracerData, TracerImageBindGroup, TracerObject, TracerPipeline, TracerRenderTextures, TracerState,
		TracerUniforms,
	},
};

pub struct TracerPipelinePlugin;

impl Plugin for TracerPipelinePlugin
{
	fn build(&self, app: &mut bevy::app::App)
	{
		app.add_plugins((
			ExtractResourcePlugin::<TracerRenderTextures>::default(),
			ExtractResourcePlugin::<TracerUniforms>::default(),
			ExtractResourcePlugin::<TracerData>::default(),
		));
		let render_app = app.sub_app_mut(RenderApp);
		render_app
			.init_resource::<TracerState>()
			.add_systems(RenderStartup, init_tracer_pipeline)
			.add_systems(Render, prepare_bind_group.in_set(RenderSystems::PrepareBindGroups))
			.add_systems(Render, update_render.in_set(RenderSystems::Prepare))
			.add_systems(RenderGraph, update_graph.before(camera_driver));
	}
}

fn init_tracer_pipeline(mut commands: Commands, asset_server: Res<AssetServer>, pipeline_cache: Res<PipelineCache>)
{
	info!("Init Pipeline");
	let bind_group_layout = BindGroupLayoutDescriptor::new(
		"Tracer",
		&BindGroupLayoutEntries::sequential(
			ShaderStages::COMPUTE,
			(
				texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
				texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadWrite),
				storage_buffer_read_only::<TracerObject>(false),
				// texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::ReadOnly),
				uniform_buffer::<TracerUniforms>(false),
			),
		),
	);
	let shader = asset_server.load(SHADER_ASSET_PATH);

	let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		layout: vec![bind_group_layout.clone()],
		shader: shader.clone(),
		entry_point: Some(Cow::from("init")),
		..default()
	});
	let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
		layout: vec![bind_group_layout.clone()],
		shader,
		entry_point: Some(Cow::from("update")),
		..default()
	});

	commands.insert_resource(TracerPipeline {
		init_pipeline,
		update_pipeline,
		texture_bind_group_layout: bind_group_layout,
	});
}

fn prepare_bind_group(
	mut commands: Commands,
	pipeline: Res<TracerPipeline>,
	gpu_images: Res<RenderAssets<GpuImage>>,
	tracer_images: Res<TracerRenderTextures>,
	tracer_uniforms: Res<TracerUniforms>,
	tracer_data: Res<TracerData>,
	render_device: Res<RenderDevice>,
	pipeline_cache: Res<PipelineCache>,
	queue: Res<RenderQueue>,
)
{
	let view_1 = gpu_images.get(&tracer_images.render_tex_1).unwrap();
	let view_2 = gpu_images.get(&tracer_images.render_tex_2).unwrap();

	let mut uniform_buffer = UniformBuffer::from(tracer_uniforms.into_inner());
	uniform_buffer.write_buffer(&render_device, &queue);

	let mut storage_buffer = StorageBuffer::from(tracer_data.0.clone());
	storage_buffer.write_buffer(&render_device, &queue);

	let bind_group_1 = render_device.create_bind_group(
		None,
		&pipeline_cache.get_bind_group_layout(&pipeline.texture_bind_group_layout),
		&BindGroupEntries::sequential((
			&view_1.texture_view,
			&view_2.texture_view,
			&storage_buffer,
			&uniform_buffer,
		)),
	);

	let bind_group_2 = render_device.create_bind_group(
		None,
		&pipeline_cache.get_bind_group_layout(&pipeline.texture_bind_group_layout),
		&BindGroupEntries::sequential((
			&view_2.texture_view,
			&view_1.texture_view,
			&storage_buffer,
			&uniform_buffer,
		)),
	);
	commands.insert_resource(TracerImageBindGroup([bind_group_1, bind_group_2]));
}

fn update_render(pipeline: Res<TracerPipeline>, pipeline_cache: Res<PipelineCache>, mut state: ResMut<TracerState>)
{
	match *state
	{
		TracerState::Loading =>
		{
			match pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
			{
				CachedPipelineState::Ok(_) =>
				{
					info!("Moving to Init");
					*state = TracerState::Init;
				}
				// If the shader hasn't loaded yet, just wait.
				CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(_)) =>
				{}
				CachedPipelineState::Err(err) =>
				{
					panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
				}
				_ =>
				{}
			}
		}
		TracerState::Init =>
		{
			if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
			{
				info!("Moving to Update");
				*state = TracerState::Update(1);
			}
		}
		TracerState::Update(0) => *state = TracerState::Update(1),
		TracerState::Update(1) => *state = TracerState::Update(0),
		TracerState::Update(_) => unreachable!(),
	}
}

fn update_graph(
	mut render_context: RenderContext,
	bind_groups: Res<TracerImageBindGroup>,
	pipeline_cache: Res<PipelineCache>,
	pipeline: Res<TracerPipeline>,
	state: Res<TracerState>,
)
{
	let mut pass = render_context
		.command_encoder()
		.begin_compute_pass(&ComputePassDescriptor::default());

	match *state
	{
		TracerState::Loading =>
		{}
		TracerState::Init =>
		{
			let init_pipeline = pipeline_cache.get_compute_pipeline(pipeline.init_pipeline).unwrap();
			pass.set_bind_group(0, &bind_groups.0[0], &[]);
			pass.set_pipeline(init_pipeline);
			pass.dispatch_workgroups(SIZE.x / WORKGROUP_SIZE, SIZE.y / WORKGROUP_SIZE, 1);
		}
		TracerState::Update(idx) =>
		{
			if let Some(update_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.update_pipeline)
			{
				pass.set_bind_group(0, &bind_groups.0[idx], &[]);
				pass.set_pipeline(update_pipeline);
				pass.dispatch_workgroups(SIZE.x / WORKGROUP_SIZE, SIZE.y / WORKGROUP_SIZE, 1);
			}
		}
	}
}
