use bevy::{
	material::descriptor::BindGroupLayoutDescriptor,
	prelude::*,
	render::{
		extract_resource::ExtractResource,
		render_resource::{BindGroup, CachedComputePipelineId, ShaderType},
	},
};

#[derive(Resource)]
pub struct TracerPipeline
{
	pub texture_bind_group_layout: BindGroupLayoutDescriptor,
	pub init_pipeline: CachedComputePipelineId,
	pub update_pipeline: CachedComputePipelineId,
}

#[derive(Resource, Clone, ExtractResource, ShaderType, Default)]
pub struct TracerUniforms
{
	pub sky_color: LinearRgba,
	pub world_from_clip: Mat4,
	pub world_position: Vec3,
	pub sun_dir: Vec3,
}

#[derive(Resource)]
pub struct TracerImageBindGroup(pub [BindGroup; 2]);

#[derive(Resource, ExtractResource, Clone)]
pub struct TracerRenderTextures
{
	pub render_tex_1: Handle<Image>,
	pub render_tex_2: Handle<Image>,
	// pub skybox: Handle<Image>,
}

#[derive(Resource, Default)]
pub enum TracerState
{
	#[default]
	Loading,
	Init,
	Update(usize),
}

#[derive(Clone, Reflect, ShaderType, Default, Debug)]
pub struct TracerObject
{
	pub obj_typef: u32,
	pub position: Vec3,
	pub rotation: Vec4,
	pub scale: Vec3,
	pub mass: f32,
}

#[derive(Resource, ExtractResource, Reflect, Debug, Default, Clone)]
pub struct TracerData(pub Vec<TracerObject>);
