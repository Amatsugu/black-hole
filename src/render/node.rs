use bevy::{
	prelude::*,
	render::{
		render_graph::{self},
		render_resource::{CachedPipelineState, ComputePassDescriptor, PipelineCache, PipelineCacheError},
		renderer::RenderContext,
	},
};

use crate::render::pipeline::{TracerImageBindGroups, TracerPipeline, TracerRenderTextures};
use crate::{SHADER_ASSET_PATH, WORKGROUP_SIZE};

pub enum TracerState {
	Loading,
	Init,
	Update(usize),
}

pub struct TracerNode {
	state: TracerState,
}

impl Default for TracerNode {
	fn default() -> Self {
		Self {
			state: TracerState::Loading,
		}
	}
}

impl render_graph::Node for TracerNode {
	fn update(&mut self, world: &mut World) {
		let pipeline = world.resource::<TracerPipeline>();
		let pipeline_cache = world.resource::<PipelineCache>();

		// if the corresponding pipeline has loaded, transition to the next stage
		match self.state {
			TracerState::Loading => {
				let shader_loaded = match pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline) {
					CachedPipelineState::Ok(_) => true,
					// If the shader hasn't loaded yet, just wait.
					CachedPipelineState::Err(PipelineCacheError::ShaderNotLoaded(_)) => false,
					CachedPipelineState::Err(err) => {
						panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
					}
					_ => false,
				};

				let tex = world.resource::<TracerRenderTextures>();
				let asset_server = world.resource::<AssetServer>();
				let load_state = asset_server.get_load_state(tex.skybox.id()).unwrap();
				if load_state.is_loaded() && shader_loaded {
					self.state = TracerState::Init;
				}
			}
			TracerState::Init => {
				if let CachedPipelineState::Ok(_) = pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
				{
					self.state = TracerState::Update(1);
				}
			}
			TracerState::Update(0) => {
				self.state = TracerState::Update(1);
			}
			TracerState::Update(1) => {
				self.state = TracerState::Update(0);
			}
			TracerState::Update(_) => unreachable!(),
		}
	}

	fn run(
		&self,
		_graph: &mut render_graph::RenderGraphContext,
		render_context: &mut RenderContext,
		world: &World,
	) -> Result<(), render_graph::NodeRunError> {
		let bind_groups = &world.resource::<TracerImageBindGroups>().0;
		let pipeline_cache = world.resource::<PipelineCache>();
		let pipeline = world.resource::<TracerPipeline>();

		let mut pass = render_context
			.command_encoder()
			.begin_compute_pass(&ComputePassDescriptor::default());

		// select the pipeline based on the current state
		match self.state {
			TracerState::Loading => {}
			TracerState::Init => {
				let init_pipeline = pipeline_cache.get_compute_pipeline(pipeline.init_pipeline).unwrap();
				pass.set_bind_group(0, &bind_groups[0], &[]);
				pass.set_pipeline(init_pipeline);
				pass.dispatch_workgroups(1920 / WORKGROUP_SIZE, 1080 / WORKGROUP_SIZE, 1);
			}
			TracerState::Update(index) => {
				if let Some(update_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.update_pipeline) {
					pass.set_bind_group(0, &bind_groups[index], &[]);
					pass.set_pipeline(update_pipeline);
					pass.dispatch_workgroups(1920 / WORKGROUP_SIZE, 1080 / WORKGROUP_SIZE, 1);
				}
			}
		}

		Ok(())
	}
}
