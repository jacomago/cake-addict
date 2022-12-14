use crate::cleanup::cleanup_components;
use crate::components::map_position::MapPosition;
use crate::config::{NPCSettings, NPCsSettings, Settings};
use crate::entities::quest::spawn_quest;
use crate::entities::RESPAWN_LABEL;
use crate::loading::TextureAtlasAssets;
use crate::map::map_builder::MapBuilder;
use crate::map::GEN_MAP_LABEL;
use crate::stages::TurnState;
use crate::systems::random_actor::RandomMover;
use crate::GameState;

use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng, RngComponent};
use iyes_loopless::prelude::IntoConditionalSystem;

use super::{ActorBundle, MapLevel};

pub struct NPCsPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for NPCsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Playing).with_system(spawn_npcs))
            .add_system_set(
                SystemSet::on_update(GameState::Playing).with_system(
                    spawn_npcs
                        .run_if_resource_equals(TurnState::NextLevel)
                        .label(RESPAWN_LABEL)
                        .after(GEN_MAP_LABEL),
                ),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Playing).with_system(
                    cleanup_components::<Npc>
                        .run_if_resource_equals(TurnState::NextLevel)
                        .before(GEN_MAP_LABEL),
                ),
            )
            .add_system_set(
                SystemSet::on_exit(GameState::Playing).with_system(cleanup_components::<Npc>),
            );
    }
}
#[derive(Component, Default)]
pub struct Npc;

#[derive(Component)]
pub struct AvailableQuest(pub Entity);

#[derive(Bundle, Default)]
pub struct NPCBundle {
    _m: Npc,
    #[bundle]
    actor: ActorBundle,
}

fn spawn_npcs(
    mut commands: Commands,
    textures: Res<TextureAtlasAssets>,
    map_builder: Res<MapBuilder>,
    mut rng: ResMut<GlobalRng>,
    settings: Res<Settings>,
    map_level: Query<&MapLevel>,
) {
    let npc_settings = &settings.npcs_settings;
    map_builder.npc_spawns.iter().for_each(|position| {
        let rng_comp = RngComponent::from(&mut rng);
        spawn_npc(
            &mut commands,
            *position,
            &textures,
            rng_comp,
            npc_settings,
            settings.tile_size,
            settings.entity_z_level,
            match map_level.get_single() {
                Ok(res) => res.value,
                Err(_) => 0,
            },
        );
    });
}

fn weights(setting: &&NPCSettings) -> f64 {
    0.01 * setting.proportion
}

fn spawn_npc(
    commands: &mut Commands,
    position: MapPosition,
    textures: &Res<TextureAtlasAssets>,
    mut rng: RngComponent,
    settings: &NPCsSettings,
    tile_size: i32,
    z_level: f32,
    map_level: u32,
) {
    let level_npcs = &settings
        .npcs
        .iter()
        .filter(|s| s.actor.entity.levels.contains(&map_level))
        .collect::<Vec<_>>();
    let config = rng.weighted_sample(level_npcs, weights).unwrap();

    let quest = config
        .quest
        .as_ref()
        .map(|settings| spawn_quest(commands, settings));
    let mut npc = commands.spawn(NPCBundle {
        actor: ActorBundle::from_settings(
            &config.actor,
            position,
            &textures.texture_atlas,
            z_level,
            tile_size,
        ),
        ..default()
    });
    npc.insert(RandomMover { rng });
    if let Some(q) = quest {
        npc.insert(AvailableQuest(q));
    };
}
