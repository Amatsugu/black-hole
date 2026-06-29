use bevy::{
	camera::visibility::RenderLayers,
	prelude::*,
	render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

use crate::{
	components::rt::{RTCamera, RTDisplay},
	render::{tracer::TracerPlugin, tracer_material::TracerMaterial},
};

pub struct Blackhole;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AssetLoad
{
	#[default]
	Pending,
	Loading,
	Init,
	Ready,
}

#[derive(Debug, Resource)]
struct SkyboxAsset(Handle<Image>);

impl Plugin for Blackhole
{
	fn build(&self, app: &mut App)
	{
		app.init_state::<AssetLoad>();
		app.add_systems(Startup, setup)
			.add_systems(Update, asset_load_check.run_if(in_state(AssetLoad::Loading)))
			.add_systems(Update, prepare_skybox.run_if(in_state(AssetLoad::Init)))
			.add_systems(Last, asset_init.run_if(in_state(AssetLoad::Init)));
		app.add_plugins((TracerPlugin, DiagnosticsOverlayPlugin ));
	}
}

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut load_state: ResMut<NextState<AssetLoad>>,
	mut materials: ResMut<Assets<TracerMaterial>>,
	mut meshes: ResMut<Assets<Mesh>>,
)
{
	commands.spawn(DiagnosticsOverlay::fps());
	let skybox_asset = asset_server.load("sky-array.png");
	commands.spawn((
		Name::new("Render Display"),
		MeshMaterial2d(materials.add(TracerMaterial {
			sky_color: LinearRgba::rgb(0.1, 0.0, 0.01),
			..default()
		})),
		RTDisplay,
		Mesh2d(meshes.add(Rectangle::from_size(vec2(1920.0, 1080.0)))),
		Transform::from_translation(Vec3::ZERO),
	));

	commands.spawn((Camera2d, RenderLayers::layer(0)));

	commands
		.spawn((
			Camera3d::default(),
			RTCamera,
			RenderLayers::layer(1),
			Transform::from_xyz(0.0, 5.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
			Name::new("RT Camera"),
		))
		.insert(Camera { order: -1, ..default() });

	commands.insert_resource(SkyboxAsset(skybox_asset));

	load_state.set(AssetLoad::Loading);
}

fn asset_load_check(
	mut load_state: ResMut<NextState<AssetLoad>>,
	skybox: Res<SkyboxAsset>,
	asset_server: Res<AssetServer>,
)
{
	let skybox_load_state = asset_server.load_state(skybox.0.id());
	if skybox_load_state.is_loaded()
	{
		load_state.set(AssetLoad::Init);
		info!("Assets Loaded");
	}
}

fn prepare_skybox(
	skybox: Res<SkyboxAsset>,
	mut image_assets: ResMut<Assets<Image>>,
	display: Single<&MeshMaterial2d<TracerMaterial>, With<RTDisplay>>,
	mut materials: ResMut<Assets<TracerMaterial>>,
)
{
	let mut skybox_image = image_assets
		.get_mut(skybox.0.id())
		.expect("Skybox asset image does not exist")
		.clone();
	skybox_image
		.reinterpret_stacked_2d_as_array(skybox_image.height() / skybox_image.width())
		.expect("Failed to re-interpret skybox");
	// skybox_image.texture_view_descriptor = Some(TextureViewDescriptor {
	// 	dimension: Some(TextureViewDimension::Cube),
	// 	..default()
	// });
	let mut mat = materials
		.get_mut(display.0.id())
		.expect("Tracer Materials doesn't exist");
	mat.skybox = Some(skybox.0.clone());
}

fn asset_init(mut load_state: ResMut<NextState<AssetLoad>>)
{
	load_state.set(AssetLoad::Ready);
	info!("Assets Initialized");
}
