use bevy::prelude::*;

use crate::{
	components::rt::{RTCamera, RTDisplay, RTHidden, RTMass, RTObject},
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
	const MOVE_SPEED: f32 = 2.0;
	let mut move_vec = Vec3::ZERO;
	if input.pressed(KeyCode::KeyW)
	{
		move_vec.z -= MOVE_SPEED;
	}
	else if input.pressed(KeyCode::KeyS)
	{
		move_vec.z += MOVE_SPEED;
	}

	if input.pressed(KeyCode::KeyA)
	{
		move_vec.x -= MOVE_SPEED
	}
	else if input.pressed(KeyCode::KeyD)
	{
		move_vec.x += MOVE_SPEED;
	}

	if input.pressed(KeyCode::ShiftLeft)
	{
		move_vec.y -= MOVE_SPEED
	}
	else if input.pressed(KeyCode::Space)
	{
		move_vec.y += MOVE_SPEED;
	}

	let t = cam.rotation * move_vec;
	cam.translation += t * time.delta_secs();
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

fn collect_data(
	mut data: ResMut<TracerData>,
	objects: Query<(&GlobalTransform, &RTMass, Option<&RTHidden>), With<RTObject>>,
)
{
	let buffer = objects.iter().map(|(t, m, hidden)| TracerObject {
		position: t.translation(),
		scale: if hidden.is_some() { 0.0 } else { t.scale().x },
		rotation: t.rotation().into(),
		mass: m.0,
		sw_radius: calculate_swr(m.0),
	});
	data.0 = buffer.collect();
}

const G: f32 = 6.674e-11;
const C: f32 = 2.998e8;
const C2: f32 = C * C;
fn calculate_swr(mass: f32) -> f32
{
	return (2.0 * G * mass) / C2;
}
