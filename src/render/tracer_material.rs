use bevy::{
	prelude::*,
	reflect::Reflect,
	render::render_resource::{AsBindGroup, ShaderType},
	sprite_render::Material2d,
};

use crate::SHADER_ASSET_PATH;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct TracerMaterial {
	#[uniform(0)]
	pub sky_color: LinearRgba,
	#[uniform(1)]
	pub view: TracerView,
	#[texture(2)]
	#[sampler(3)]
	pub skybox: Option<Handle<Image>>,
	// #[uniform(4)]
	// pub objects: TracerObjects,
}

#[derive(Debug, ShaderType, Clone, Reflect, Default)]
pub struct TracerView {
	pub world_from_clip: Mat4,
	pub world_position: Vec3,
}

impl Material2d for TracerMaterial {
	fn fragment_shader() -> bevy::shader::ShaderRef {
		SHADER_ASSET_PATH.into()
	}
}

/*
#[derive(Debug, ShaderType, Clone, Reflect, Default)]
pub struct TracerObjects {
	pub objects: Vec<Object>,
}
#[derive(Debug, ShaderType, Clone, Reflect, Default)]
pub struct Object {
	pub object_type: u32,
	pub position: Vec3,
	pub rotation: Vec4,
	pub scale: Vec3,
}
*/
