use bevy::{
	prelude::*,
	render::{
		extract_resource::ExtractResource,
		render_resource::{BindGroup, BindGroupLayout, CachedComputePipelineId, ShaderType},
	},
};

#[derive(Resource)]
pub struct TracerPipeline
{
	pub texture_bind_group_layout: BindGroupLayout,
	pub init_pipeline: CachedComputePipelineId,
	pub update_pipeline: CachedComputePipelineId,
}

#[derive(Resource, Clone, ExtractResource, ShaderType, Default)]
pub struct TracerUniforms
{
	pub sky_color: LinearRgba,
	pub world_from_clip: Mat4,
	pub world_position: Vec3,
}

#[derive(Resource)]
pub struct TracerImageBindGroups(pub [BindGroup; 2]);

#[derive(Resource, Reflect, ExtractResource, Clone)]
#[reflect(Resource)]
pub struct TracerRenderTextures
{
	pub main: Handle<Image>,
	pub secondary: Handle<Image>,
	pub skybox: Handle<Image>,
}
