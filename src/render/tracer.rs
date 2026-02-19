use bevy::{prelude::*, sprite_render::Material2dPlugin};

use crate::{
	app::AssetLoad,
	components::rt::{RTCamera, RTDisplay},
	render::tracer_material::{TracerMaterial, TracerView},
};

pub struct TracerPlugin;

impl Plugin for TracerPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(Material2dPlugin::<TracerMaterial>::default());
		app.add_systems(First, update_tracer_uniforms.run_if(in_state(AssetLoad::Ready)));
	}
}

fn update_tracer_uniforms(
	rt_camera: Single<(&GlobalTransform, &Camera), With<RTCamera>>,
	display: Single<&MeshMaterial2d<TracerMaterial>, With<RTDisplay>>,
	mut materials: ResMut<Assets<TracerMaterial>>,
) {
	let (transform, cam) = rt_camera.into_inner();

	let clip_from_view = cam.clip_from_view();
	let world_from_clip = transform.to_matrix() * clip_from_view.inverse();

	let mat = materials
		.get_mut(display.0.id())
		.expect("Tracer Materials doesn't exist");
	mat.view = TracerView {
		world_from_clip: world_from_clip,
		world_position: transform.translation(),
	};
}
