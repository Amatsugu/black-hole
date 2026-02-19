use bevy::{
	asset::RenderAssetUsages,
	prelude::*,
	render::{
		render_resource::{
			Extent3d, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension,
		},
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

#[derive(Debug, Resource)]
struct SkyboxAsset(Handle<Image>);

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

	let render0 = images.add(image.clone());
	let render1 = images.add(image);

	let mut skybox_render_image = Image::new_fill(
		Extent3d {
			width: 256,
			height: 1536,
			..Default::default()
		},
		TextureDimension::D2,
		&[255; PIXEL_SIZE],
		PIXEL_FORMAT,
		RenderAssetUsages::RENDER_WORLD,
	);
	skybox_render_image.reinterpret_stacked_2d_as_array(6);
	skybox_render_image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
	skybox_render_image.texture_view_descriptor = Some(TextureViewDescriptor {
		dimension: Some(TextureViewDimension::Cube),
		..default()
	});
	let skybox_render_image_handle = images.add(skybox_render_image);

	let skybox_asset = asset_server.load("sky-array.png");
	commands.spawn((
		Name::new("Render Sprite"),
		Sprite {
			image: render0.clone(),
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
		main: render0,
		secondary: render1,
		skybox: skybox_render_image_handle,
	});
	commands.insert_resource(SkyboxAsset(skybox_asset));

	load_state.set(AssetLoad::Loading);
}

fn asset_load_check(
	mut load_state: ResMut<NextState<AssetLoad>>,
	skybox: Res<SkyboxAsset>,
	asset_server: Res<AssetServer>,
) {
	let skybox_load_state = asset_server.load_state(skybox.0.id());
	if skybox_load_state.is_loaded() {
		load_state.set(AssetLoad::Init);
		info!("Assets Loaded");
	}
}

fn prepare_skybox(
	tracer_textures: Res<TracerRenderTextures>,
	skybox: Res<SkyboxAsset>,
	mut image_assets: ResMut<Assets<Image>>,
) {
	let mut skybox_image = image_assets
		.get(skybox.0.id())
		.expect("Skybox asset image does not exist")
		.clone();
	skybox_image.reinterpret_stacked_2d_as_array(skybox_image.height() / skybox_image.width());
	skybox_image.texture_view_descriptor = Some(TextureViewDescriptor {
		dimension: Some(TextureViewDimension::Cube),
		..default()
	});
	image_assets.insert(tracer_textures.skybox.id(), skybox_image.clone());
}

fn asset_init(mut load_state: ResMut<NextState<AssetLoad>>) {
	load_state.set(AssetLoad::Ready);
	info!("Assets Initialized");
}
