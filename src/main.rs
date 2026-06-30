use app::Blackhole;
use bevy::window::PresentMode;
use bevy::{prelude::*, window::WindowResolution};

mod app;
pub mod components;
mod render;

pub const SHADER_ASSET_PATH: &str = "trace-compute.wgsl";
pub const WORKGROUP_SIZE: u32 = 8;
pub const SIZE: UVec2 = UVec2::new(1920, 1080);
const NAME: &str = "Black Hole";

fn main()
{
	App::new()
		.add_plugins((
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						title: NAME.into(),
						name: Some(NAME.into()),
						resolution: WindowResolution::new(SIZE.x, SIZE.y),
						present_mode: PresentMode::AutoNoVsync,
						resizable: false,
						..default()
					}),
					..default()
				})
				.set(AssetPlugin {
					#[cfg(not(debug_assertions))]
					watch_for_changes_override: Some(true),
					..Default::default()
				}),
			Blackhole,
			// EguiPlugin::default(),
		))
		.run();
}
