//! Cake Addict is a rogue like game made in bevy. Inspired by
//! the health industry.

//#![warn(clippy::pedantic)] // turn on for extra hints
#![deny(
    missing_docs,
    rustdoc::missing_doc_code_examples,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    variant_size_differences
)]
//#![forbid(clippy::missing_docs_in_private_items)]

mod actions;
mod audio;
mod camera;
mod cleanup;
mod components;
mod config;
mod entities;
mod game_ui;
mod loading;
mod map;
mod menu;
mod stages;
mod systems;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::config::ConfigPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use camera::CameraPlugin;
use entities::EntitiesPlugin;
use game_ui::GameUiPlugin;
use map::MapPlugin;
use stages::StagePlugin;
use systems::SystemsPlugin;

/// This game uses States to separate logic
/// See https://bevy-cheatbook.github.io/programming/states.html
/// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    /// During the loading State the LoadingPlugin will load our assets
    Loading,
    /// Generate procedural objects
    Generation,
    /// During this State the actual game logic is executed
    Playing,
    /// Here the menu is drawn and waiting for player interaction
    Menu,
}

/// Main plugin for the games internals
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state(GameState::Loading)
            .add_plugin(ConfigPlugin)
            .add_plugin(LoadingPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(ActionsPlugin)
            .add_plugin(InternalAudioPlugin)
            .add_plugin(StagePlugin)
            .add_plugin(SystemsPlugin)
            .add_plugin(GameUiPlugin)
            .add_plugin(CameraPlugin)
            .add_plugin(MapPlugin)
            .add_plugin(EntitiesPlugin);

        #[cfg(debug_assertions)]
        {
            app.add_plugin(LogDiagnosticsPlugin::default())
                .add_plugin(WorldInspectorPlugin::new())
                .add_system(bevy::window::close_on_esc);
        }
    }
}
