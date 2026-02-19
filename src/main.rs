use app::Blackhole;
use bevy::window::PresentMode;
use bevy::{prelude::*, window::WindowResolution};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod app;
pub mod components;
mod render;

pub const SHADER_ASSET_PATH: &str = "trace.wgsl";
pub const WORKGROUP_SIZE: u32 = 8;
const NAME: &str = "Black Hole";

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						title: NAME.into(),
						name: Some(NAME.into()),
						resolution: WindowResolution::new(1920, 1080),
						present_mode: PresentMode::AutoNoVsync,
						..default()
					}),
					..default()
				})
				.set(AssetPlugin {
					#[cfg(not(debug_assertions))]
					watch_for_changes_override: Some(true),
					..Default::default()
				}),
			EguiPlugin::default(),
			WorldInspectorPlugin::new(),
			Blackhole,
		))
		.run();
}
