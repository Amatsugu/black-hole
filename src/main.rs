use app::Blackhole;
use bevy::prelude::*;
use bevy::window::PresentMode;
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
						resolution: (1920., 1080.).into(),
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
			EguiPlugin {
				enable_multipass_for_primary_context: true,
			},
			WorldInspectorPlugin::new(),
			Blackhole,
		))
		.run();
}
