use bevy::{
	asset::RenderAssetUsages,
	math::VectorSpace,
	prelude::*,
	render::{
		render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
		view::RenderLayers,
	},
	window::PrimaryWindow,
};
use iyes_perf_ui::prelude::*;

use crate::{
	components::rt::RTCamera,
	render::pipeline::{TracerPipelinePlugin, TracerRenderTextures, TracerUniforms},
};

pub struct Blackhole;

impl Plugin for Blackhole {
	fn build(&self, app: &mut App) {
		app.register_type::<TracerRenderTextures>();

		app.add_systems(Startup, setup);
		app.add_plugins(TracerPipelinePlugin);
		app.insert_resource(TracerUniforms {
			sky_color: LinearRgba::rgb(0.1, 0.0, 0.01),
			..default()
		});

		//Perf UI
		app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
			.add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
			.add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
			.add_plugins(PerfUiPlugin);
	}
}

fn setup(
	mut commands: Commands,
	mut images: ResMut<Assets<Image>>,
	window: Single<&Window, With<PrimaryWindow>>,
	asset_server: Res<AssetServer>,
) {
	commands.spawn((
		PerfUiRoot::default(),
		PerfUiEntryFPS::default(),
		PerfUiEntryFPSWorst::default(),
		PerfUiEntryFrameTime::default(),
		PerfUiEntryFrameTimeWorst::default(),
	));

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

	let skybox = asset_server.load("sky-test.png");
	commands.spawn((
		Name::new("Render Sprite"),
		Sprite {
			image: img0.clone(),
			custom_size: Some(size.as_vec2()),
			..default()
		},
		Transform::from_translation(Vec3::ZERO),
	));

	commands.spawn((Camera2d, RenderLayers::layer(0)));

	commands
		.spawn((
			Camera3d::default(),
			// Camera { order: -1, ..default() },
			// Projection::Perspective(PerspectiveProjection {
			// 	aspect_ratio: size.x as f32 / size.y as f32,
			// 	..default()
			// }),
			RTCamera,
			RenderLayers::layer(1),
			Transform::from_xyz(0.0, 5.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
			Name::new("RT Camera"),
		))
		.insert(Camera { order: -1, ..default() });

	commands.insert_resource(TracerRenderTextures {
		main: img0,
		secondary: img1,
		skybox,
	});
}
