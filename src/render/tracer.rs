use bevy::prelude::*;

use crate::{
	components::rt::{RTCamera, RTDisplay},
	render::{
		pipeline::TracerPipelinePlugin,
		resources::{TracerRenderTextures, TracerUniforms},
	},
};

pub struct TracerPlugin;

impl Plugin for TracerPlugin
{
	fn build(&self, app: &mut App)
	{
		app.add_plugins(TracerPipelinePlugin);
		app.add_systems(Update, (update_uniforms, swap_image, cam_movement));
	}
}

fn update_uniforms(
	cam: Single<(&Transform, &Projection), With<RTCamera>>,
	mut tracer_uniforms: ResMut<TracerUniforms>,
	sun: Single<&Transform, With<DirectionalLight>>,
)
{
	let (transform, cam) = cam.into_inner();

	let clip_from_view = cam.get_clip_from_view();
	let world_from_clip = transform.to_matrix() * clip_from_view.inverse();

	tracer_uniforms.world_from_clip = world_from_clip;
	tracer_uniforms.world_position = transform.translation;
	tracer_uniforms.sun_dir = sun.forward().as_vec3();
}

fn cam_movement(mut cam: Single<&mut Transform, With<RTCamera>>, time: Res<Time>)
{
	cam.translation.y = f32::sin(time.elapsed_secs()) + 5.;
}

fn swap_image(mut display: Single<&mut ImageNode, With<RTDisplay>>, images: Res<TracerRenderTextures>)
{
	if display.image == images.render_tex_1
	{
		display.image = images.render_tex_2.clone();
	}
	else
	{
		display.image = images.render_tex_1.clone();
	}
}
