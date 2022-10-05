use crate::actions::Actions;
use crate::loading::TextureAtlasAssets;
use crate::map::map_builder::MapBuilder;
use crate::map::map_position::MapPosition;
use crate::stages::{end_turn, GameStage, TurnState};
use crate::systems::collisions;
use crate::GameState;

use bevy::prelude::*;
use iyes_loopless::prelude::*;

const PLAYER_SPRITE_INDEX: usize = 64;
const PLAYER_Z: f32 = 1.;

pub struct PlayerPlugin;

#[derive(Component, Default)]
pub struct Player;

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    _player: Player,
    pub position: MapPosition,
    #[bundle]
    sprite: SpriteSheetBundle,
}

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_player))
            .add_system_set(
                SystemSet::on_update(GameState::Playing)
                    .with_system(move_player.run_if_resource_equals(TurnState::AwaitingInput))
                    .with_system(focus_camera),
            )
            .add_system_set_to_stage(
                GameStage::MovePlayer,
                ConditionSet::new()
                    .run_if_resource_equals(TurnState::PlayerTurn)
                    .with_system(collisions::collisions)
                    .with_system(end_turn)
                    .into(),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    textures: Res<TextureAtlasAssets>,
    map_builder: Res<MapBuilder>,
) {
    let player_start = map_builder.player_start;
    commands.spawn_bundle(PlayerBundle {
        position: player_start,
        sprite: SpriteSheetBundle {
            transform: Transform {
                translation: player_start.translation(PLAYER_Z),
                ..default()
            },
            texture_atlas: textures.texture_atlas.clone(),
            sprite: TextureAtlasSprite {
                index: PLAYER_SPRITE_INDEX,
                ..default()
            },
            ..default()
        },
        ..default()
    });
}

fn move_player(
    mut commands: Commands,
    actions: Res<Actions>,
    map_builder: Res<MapBuilder>,
    mut player_query: Query<(&mut Transform, &mut MapPosition, With<Player>)>,
) {
    if actions.player_movement.is_none() {
        return;
    }
    let movement = actions.player_movement.unwrap().as_ivec2();
    let (mut transform, mut position, _) = player_query.single_mut();
    let new_position = MapPosition::from_ivec2(position.position + movement);
    if map_builder.map.can_enter_tile(new_position) {
        transform.translation = new_position.translation(PLAYER_Z);
        info!(
            "Player moved to {} from {}",
            position.position, new_position.position
        );
        position.position = new_position.position;
    }

    commands.insert_resource(TurnState::PlayerTurn);
}

// change the focus of the camera to the player
pub fn focus_camera(
    mut transforms: ParamSet<(
        Query<&mut Transform, With<Camera2d>>,
        Query<&Transform, With<Player>>,
    )>,
) {
    let binding = transforms.p1();
    let translation = binding.single().translation;
    let mut binding = transforms.p0();
    let mut camera_transform = binding.single_mut();
    *camera_transform = Transform::from_translation(translation);
}