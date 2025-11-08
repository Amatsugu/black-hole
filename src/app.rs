use std::default;

use bevy::{
	asset::RenderAssetUsages,
	image::ImageSamplerDescriptor,
	prelude::*,
	render::{
		render_resource::{Extent3d, SamplerDescriptor, TextureDimension, TextureFormat, TextureUsages},
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

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AssetLoad {
	#[default]
	Pending,
	Loading,
	Init,
	Ready,
}

impl Plugin for Blackhole {
	fn build(&self, app: &mut App) {
		app.register_type::<TracerRenderTextures>();

		app.init_state::<AssetLoad>();
		app.add_systems(Startup, setup)
			.add_systems(Update, asset_load_check.run_if(in_state(AssetLoad::Loading)))
			.add_systems(Update, prepare_skybox.run_if(in_state(AssetLoad::Init)))
			.add_systems(Last, asset_init.run_if(in_state(AssetLoad::Init)));
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
	mut load_state: ResMut<NextState<AssetLoad>>,
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

	load_state.set(AssetLoad::Loading);
}

fn asset_load_check(
	mut load_state: ResMut<NextState<AssetLoad>>,
	tracer_textures: Res<TracerRenderTextures>,
	asset_server: Res<AssetServer>,
) {
	let skybox_load_state = asset_server.load_state(tracer_textures.skybox.id());
	if skybox_load_state.is_loaded() {
		load_state.set(AssetLoad::Init);
		info!("Assets Loaded");
	}
}

fn prepare_skybox(tracer_textures: Res<TracerRenderTextures>, image_assets: Res<Assets<Image>>) {
	let sb = image_assets.get(tracer_textures.skybox.id()).unwrap();
}

fn asset_init(mut load_state: ResMut<NextState<AssetLoad>>) {
	load_state.set(AssetLoad::Ready);
	info!("Assets Initialized");
}
