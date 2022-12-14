// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::window::{WindowId, WindowPlugin};
use bevy::winit::WinitWindows;
use bevy::DefaultPlugins;
use bevy_egui::EguiPlugin;
use bevy_turborand::RngPlugin;
use cake_addict::GamePlugin;
use std::io::Cursor;
use winit::window::Icon;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;
// const RNG_SEED: u64 = 1000;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: WIDTH,
                height: HEIGHT,
                title: "Cake Addict".to_string(),
                canvas: Some("#bevy".to_owned()),
                ..default()
            },
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .add_plugin(RngPlugin::new())
        .add_plugin(GamePlugin)
        .add_startup_system(set_window_icon)
        .run();
}

// Sets the icon on windows and X11
fn set_window_icon(windows: NonSend<WinitWindows>) {
    let primary = windows.get_window(WindowId::primary()).unwrap();
    let icon_buf = Cursor::new(include_bytes!("../assets/textures/app_icon.png"));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
