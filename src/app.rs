use bevy::{
	asset::RenderAssetUsages,
	prelude::*,
	render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
	window::PrimaryWindow,
};

use crate::render::pipeline::{TracerPipelinePlugin, TracerRenderTextures, TracerUniforms};

pub struct Blackhole;

impl Plugin for Blackhole {
	fn build(&self, app: &mut App) {
		app.register_type::<TracerRenderTextures>();

		app.add_systems(Startup, setup);

		app.add_plugins(TracerPipelinePlugin);
		app.insert_resource(TracerUniforms {
			sky_color: LinearRgba::BLUE,
		});
	}
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, window: Single<&Window, With<PrimaryWindow>>) {
	let size = window.physical_size();

	let extent = Extent3d {
		width: size.x,
		height: size.y,
		..Default::default()
	};

	const PIXEL_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
	const PIXEL_SIZE: usize = 16;
	let mut image = Image::new_fill(
		extent,
		TextureDimension::D2,
		&[255; PIXEL_SIZE],
		PIXEL_FORMAT,
		RenderAssetUsages::RENDER_WORLD,
	);
	image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
		| TextureUsages::STORAGE_BINDING
		| TextureUsages::COPY_DST
		| TextureUsages::RENDER_ATTACHMENT;

	let img0 = images.add(image.clone());
	let img1 = images.add(image);

	commands.spawn((
		Name::new("Render Sprite"),
		Sprite {
			image: img0.clone(),
			custom_size: Some(size.as_vec2()),
			..default()
		},
		Transform::from_translation(Vec3::ZERO),
	));

	commands.spawn(Camera2d);

	commands.insert_resource(TracerRenderTextures(img0, img1));
}
