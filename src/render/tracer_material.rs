use bevy::{
	prelude::*,
	reflect::Reflect,
	render::render_resource::{AsBindGroup, ShaderType},
	sprite::Material2d,
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
}

#[derive(Debug, ShaderType, Clone, Reflect, Default)]
pub struct TracerView {
	pub world_from_clip: Mat4,
	pub world_position: Vec3,
}

impl Material2d for TracerMaterial {
	fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
		SHADER_ASSET_PATH.into()
	}

	fn alpha_mode(&self) -> bevy::sprite::AlphaMode2d {
		bevy::sprite::AlphaMode2d::Opaque
	}
}
