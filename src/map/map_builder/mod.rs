use std::fmt::Display;

use crate::components::map_position::MapPosition;
use crate::config::{Architect, ArchitectSettings};
use crate::entities::TileType;
use bevy::prelude::Resource;
use bevy::utils::HashSet;
use bevy_turborand::{DelegatedRng, RngComponent};

use self::automata::CellularAutomataArchitect;
use self::drunkard::DrunkardArchitect;
use self::empty::EmptyArchitect;
use self::prefab::apply_prefab;
use self::standard::StandardArchitect;

use super::grid_map::base_map::BaseMap;
use super::grid_map::DjikstraMapCalc;
use super::tile_map::TileMap;

mod automata;
mod drunkard;
mod empty;
mod prefab;
mod standard;
const MAX_ATTEMPTS: usize = 10;
trait MapArchitect {
    fn entity_distance(&self) -> f32;
    fn num_monsters(&self) -> usize;
    fn num_items(&self) -> usize;
    fn num_npcs(&self) -> usize;
    fn builder(&mut self, height: usize, width: usize, rng: &mut RngComponent) -> MapBuilder;

    fn entity_spawns(
        &self,
        start: MapPosition,
        map: &TileMap,
        rng: &mut RngComponent,
        amount: usize,
    ) -> HashSet<MapPosition> {
        let tiles = map
            .tiles
            .indexed_iter()
            .map(|(idx, t)| (MapPosition::from_utuple(&idx), t))
            .filter(|(idx, t)| {
                **t == TileType::Floor && idx.distance(start) > self.entity_distance()
            })
            .map(|(idx, _)| idx)
            .collect::<Vec<MapPosition>>();

        let spawns = rng
            .sample_multiple(&tiles, amount)
            .iter()
            .map(|f| **f)
            .collect();
        spawns
    }
}

#[derive(Debug, Default, Resource)]
pub struct MapBuilder {
    pub map: TileMap,
    pub monster_spawns: HashSet<MapPosition>,
    pub item_spawns: HashSet<MapPosition>,
    pub npc_spawns: HashSet<MapPosition>,
    pub player_start: MapPosition,
    pub winitem_start: MapPosition,
}

fn pick_architect(architect: &ArchitectSettings) -> Box<dyn MapArchitect> {
    match architect.architect {
        Architect::Empty => Box::new(EmptyArchitect::new(
            architect.num_monsters,
            architect.num_items,
            architect.num_npcs,
            architect.entity_distance,
        )),
        Architect::Standard => Box::new(StandardArchitect::new(
            architect.num_monsters,
            architect.num_items,
            architect.num_npcs,
            architect.entity_distance,
        )),
        Architect::Automata => Box::new(CellularAutomataArchitect::new(
            architect.num_monsters,
            architect.num_items,
            architect.num_npcs,
            architect.entity_distance,
        )),
        Architect::Drunkard => Box::new(DrunkardArchitect::new(
            architect.num_monsters,
            architect.num_items,
            architect.num_npcs,
            architect.entity_distance,
        )),
    }
}

impl MapBuilder {
    pub fn new(
        mut rng: RngComponent,
        height: usize,
        width: usize,
        architect: &ArchitectSettings,
    ) -> Self
    where
        Self: Sized,
    {
        let mut map_arch = pick_architect(architect);
        let mut mb = map_arch.builder(height, width, &mut rng);

        apply_prefab(&mut mb, MAX_ATTEMPTS, &mut rng, 20, 2000);
        mb
    }

    fn find_most_distant(&self) -> MapPosition {
        self.map.djikstra_map(self.player_start).furthest_point()
    }

    fn fill(&mut self, tile: TileType) {
        self.map.tiles.iter_mut().for_each(|t| *t = tile);
    }

    fn fill_in_unreachable(&mut self) {
        self.map
            .djikstra_map(self.player_start)
            .far_points(None)
            .iter()
            .for_each(|p| {
                self.map.set(*p, TileType::Wall);
            });
    }
}

impl Display for MapBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_tiles = self
            .map
            .tiles
            .rows()
            .into_iter()
            .enumerate()
            .map(|(x, row)| {
                row.iter()
                    .enumerate()
                    .map(|(y, tile)| {
                        let mp = MapPosition::from_utuple(&(x, y));
                        if self.player_start == mp {
                            "@".to_string()
                        } else if self.winitem_start == mp {
                            "?".to_string()
                        } else if self.monster_spawns.contains(&mp) {
                            "M".to_string()
                        } else if self.npc_spawns.contains(&mp) {
                            "N".to_string()
                        } else if self.item_spawns.contains(&mp) {
                            "I".to_string()
                        } else {
                            format!("{}", tile)
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<String>>()
            .join("\n");
        f.write_fmt(format_args!("{}", str_tiles))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build() {
        let rng = RngComponent::new();
        let mb = MapBuilder::new(
            rng,
            40,
            80,
            &ArchitectSettings {
                architect: Architect::Drunkard,
                num_monsters: 40,
                num_items: 10,
                num_npcs: 5,
                entity_distance: 10.0,
            },
        );
        println!("{}", mb);
    }
    #[test]
    fn gen_many() {
        (0..1000).for_each(|_| {
            let rng = RngComponent::new();
            MapBuilder::new(
                rng,
                40,
                80,
                &ArchitectSettings {
                    architect: Architect::Drunkard,
                    num_monsters: 40,
                    num_items: 10,
                    num_npcs: 5,
                    entity_distance: 10.0,
                },
            );
        });
    }
}
