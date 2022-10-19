use bevy::prelude::*;

use crate::{
    cleanup::cleanup_components,
    components::{map_position::MapPosition, name::CharacterName},
    config::Settings,
    game_ui::tooltip::Interactive,
    loading::TextureAtlasAssets,
    map::map_builder::MapBuilder,
    systems::movement::CHARACTER_Z,
    GameState,
};

pub struct ItemsPlugin;

impl Plugin for ItemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_wintitem))
            .add_system_set(
                SystemSet::on_exit(GameState::Playing).with_system(cleanup_components::<Item>),
            );
    }
}

#[derive(Component, Default)]
pub struct Item;

#[derive(Component)]
pub struct WinItem;

#[derive(Bundle, Default)]
pub struct ItemBundle {
    _i: Item,
    pub name: CharacterName,
    pub position: MapPosition,
    pub interactive: Interactive,
    #[bundle]
    sprite: SpriteSheetBundle,
}

static WINITEM_NAME: &str = "Cake of destiny";
const WINITEM_SPRITE_INDEX: usize = 124;

pub fn spawn_wintitem(
    mut commands: Commands,
    textures: Res<TextureAtlasAssets>,
    map_builder: Res<MapBuilder>,
    settings: Res<Settings>,
) {
    let position = map_builder.winitem_start;
    commands
        .spawn_bundle(ItemBundle {
            name: CharacterName(WINITEM_NAME.to_owned()),
            position,
            interactive: Interactive {
                text: WINITEM_NAME.to_string(),
            },
            sprite: SpriteSheetBundle {
                transform: Transform {
                    translation: position.translation(CHARACTER_Z, settings.tile_size),
                    ..default()
                },
                texture_atlas: textures.texture_atlas.clone(),
                sprite: TextureAtlasSprite {
                    index: WINITEM_SPRITE_INDEX,
                    ..default()
                },

                ..default()
            },
            ..default()
        })
        .insert(WinItem);
}
