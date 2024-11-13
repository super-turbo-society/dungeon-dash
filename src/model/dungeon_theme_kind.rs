use super::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DungeonThemeKind {
    Castle,
    Crypt,
    Pirate,
    Forest,
}
impl DungeonThemeKind {
    pub const KINDS: &'static [Self] = &[Self::Castle, Self::Crypt, Self::Pirate, Self::Forest];
    pub const THEMES: &'static [DungeonTheme] = &[
        DungeonTheme {
            particle_color: 0xb41c39ff,
            // particle_color: 0x000000ff,
            mist_color: 0xffffff09,
            dungeon_border: "castle_dungeon_nine_slice",
            floor_sprite: "floor",
            block_a_sprite: "wall",
            block_b_sprite: "firepit",
        },
        DungeonTheme {
            particle_color: 0x7b34bdff,
            mist_color: 0xffffff09,
            dungeon_border: "crypt_dungeon_nine_slice",
            floor_sprite: "dark_floor",
            // block_a_sprite: "metal_block",
            block_b_sprite: "metal_crate",
            // block_a_sprite: "dark_stone",
            // block_a_sprite: "necro_block",
            block_a_sprite: "tombstone2",
            // block_b_sprite: "crumbled_pillar",
        },
        DungeonTheme {
            particle_color: 0xff9e21ff,
            mist_color: 0xffffff09,
            dungeon_border: "pirate_dungeon_nine_slice",
            floor_sprite: "wood_floor",
            block_a_sprite: "crate",
            block_b_sprite: "barrel",
        },
        DungeonTheme {
            particle_color: 0x6ab2c5ff,
            mist_color: 0xffffff09,
            dungeon_border: "forest_dungeon_nine_slice",
            floor_sprite: "floor_forest",
            block_a_sprite: "shrub",
            block_b_sprite: "stump",
        },
    ];
    pub fn theme(&self) -> DungeonTheme {
        match self {
            Self::Castle => Self::THEMES[0],
            Self::Crypt => Self::THEMES[1],
            Self::Pirate => Self::THEMES[2],
            Self::Forest => Self::THEMES[3],
        }
    }
}
