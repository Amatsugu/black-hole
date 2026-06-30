use bevy::prelude::*;

use crate::{
	components::rt::{RTCamera, RTDisplay, RTMass, RTObject},
	render::{
		pipeline::TracerPipelinePlugin,
		resources::{TracerData, TracerObject, TracerRenderTextures, TracerUniforms},
	},
};

pub struct TracerPlugin;

impl Plugin for TracerPlugin
{
	fn build(&self, app: &mut App)
	{
		app.init_resource::<TracerData>().add_plugins(TracerPipelinePlugin);
		app.add_systems(Update, (update_uniforms, swap_image, cam_movement));
		app.add_systems(PostUpdate, collect_data);
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

fn cam_movement(mut cam: Single<&mut Transform, With<RTCamera>>, time: Res<Time>, input: Res<ButtonInput<KeyCode>>)
{
	let mut move_vec = Vec3::ZERO;
	if input.pressed(KeyCode::KeyW)
	{
		move_vec.z = time.delta_secs();
	}
	else if input.pressed(KeyCode::KeyS)
	{
		move_vec.z += time.delta_secs();
	}
	if input.pressed(KeyCode::KeyA)
	{
		move_vec.x -= time.delta_secs()
	}
	else if input.pressed(KeyCode::KeyD)
	{
		move_vec.x += time.delta_secs();
	}

	if input.pressed(KeyCode::ShiftLeft)
	{
		move_vec.y -= time.delta_secs()
	}
	else if input.pressed(KeyCode::Space)
	{
		move_vec.y += time.delta_secs();
	}

	let t = cam.rotation * move_vec;
	cam.translation += t;
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

fn collect_data(mut data: ResMut<TracerData>, objects: Query<(&GlobalTransform, &RTObject, &RTMass)>)
{
	let buffer = objects.iter().map(|(t, o, m)| TracerObject {
		position: t.translation(),
		scale: t.scale(),
		rotation: t.rotation().into(),
		obj_typef: o.0,
		mass: m.0,
	});
	data.0 = buffer.collect();
}
