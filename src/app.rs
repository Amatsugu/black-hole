use bevy::{
	asset::RenderAssetUsages,
	prelude::*,
	render::render_resource::{TextureFormat, TextureUsages},
};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
#[cfg(feature = "dev")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{
	SIZE,
	components::rt::{RTCamera, RTDisplay, RTHidden, RTMass, RTObject},
	render::{
		resources::{TracerRenderTextures, TracerUniforms},
		tracer::TracerPlugin,
	},
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
		app.add_systems(Startup, (setup, setup_objects))
			.add_systems(Update, asset_load_check.run_if(in_state(AssetLoad::Loading)))
			.add_systems(Update, prepare_skybox.run_if(in_state(AssetLoad::Init)))
			.add_systems(Update, rotate)
			.add_systems(Last, asset_init.run_if(in_state(AssetLoad::Init)));
		app.add_plugins((TracerPlugin, EguiPlugin::default()));
		#[cfg(feature = "dev")]
		app.add_plugins(WorldInspectorPlugin::new());
	}
}

fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut images: ResMut<Assets<Image>>,
	mut load_state: ResMut<NextState<AssetLoad>>,
)
{
	let skybox_asset = asset_server.load("sky-array.png");
	commands.insert_resource(SkyboxAsset(skybox_asset.clone()));

	let mut image = Image::new_target_texture(SIZE.x, SIZE.y, TextureFormat::Rgba32Float, None);
	image.asset_usage = RenderAssetUsages::RENDER_WORLD;
	image.texture_descriptor.usage =
		TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
	let render_image_1 = images.add(image.clone());
	let render_image_2 = images.add(image);
	commands.insert_resource(TracerRenderTextures {
		render_tex_1: render_image_1.clone(),
		render_tex_2: render_image_2,
	});

	commands.spawn((
		Name::new("Render Display"),
		Node {
			height: Val::Percent(100.),
			width: Val::Percent(100.),
			..default()
		},
		ImageNode {
			image: render_image_1,
			..default()
		},
		RTDisplay,
		Transform::from_xyz(0.0, 0.0, -1.0),
	));

	commands.spawn(Camera2d);

	commands.spawn((
		Projection::Perspective(PerspectiveProjection {
			aspect_ratio: SIZE.x as f32 / SIZE.y as f32,
			..default()
		}),
		RTCamera,
		Transform::from_xyz(0.0, 5.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
		Name::new("RT Camera"),
	));

	commands.spawn((
		DirectionalLight::default(),
		Transform::from_xyz(10., 10., 10.).looking_at(Vec3::ZERO, Vec3::Y),
	));

	load_state.set(AssetLoad::Loading);

	commands.insert_resource(TracerUniforms {
		sky_color: LinearRgba::rgb(0.0, 0.0, 0.0),
		..default()
	});
}

#[derive(Component, Reflect)]
struct Rotator(pub f32);

fn setup_objects(mut commands: Commands)
{
	commands
		.spawn((RTObject, RTMass(1e18), RTHidden, Rotator(10_f32.to_radians())))
		.with_children(|cmd| {
			for i in 0..10
			{
				let a = f32::to_radians(i as f32 * 36.0);
				cmd.spawn((
					RTObject,
					Transform::from_scale(Vec3::splat(0.2)).with_translation(Vec3::new(
						a.cos() * 2.0,
						0.0,
						a.sin() * 2.0,
					)),
				));
			}
		});
}

fn rotate(rotators: Query<(&mut Transform, &Rotator)>, time: Res<Time>)
{
	for (mut transform, rot) in rotators
	{
		transform.rotate_local_y(rot.0 * time.delta_secs());
	}
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

fn prepare_skybox(_skybox: Res<SkyboxAsset>, mut _image_assets: ResMut<Assets<Image>>) {}

fn asset_init(mut load_state: ResMut<NextState<AssetLoad>>)
{
	load_state.set(AssetLoad::Ready);
	info!("Assets Initialized");
}
