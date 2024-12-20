use super::*;

use serde::{Deserialize, Serialize};

#[derive(
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub enum DungeonThemeKind {
    Castle,
    Crypt,
    Pirate,
    Forest,
    IceCave,
    Arctic,
}
impl DungeonThemeKind {
    pub const ALL: &'static [Self] = &[
        Self::Castle,
        Self::Crypt,
        Self::Pirate,
        Self::Forest,
        Self::IceCave,
        Self::Arctic,
    ];
    pub const WINTER: &'static [Self] = &[
        Self::Castle,
        Self::Crypt,
        Self::Pirate,
        Self::Forest,
        Self::IceCave,
        Self::IceCave,
        Self::IceCave,
        Self::IceCave,
        Self::Arctic,
        Self::Arctic,
        Self::Arctic,
        Self::Arctic,
        Self::IceCave,
        Self::IceCave,
        Self::IceCave,
        Self::IceCave,
        Self::Arctic,
        Self::Arctic,
        Self::Arctic,
        Self::Arctic,
    ];
    pub const THEMES: &'static [DungeonTheme] = &[
        // Castle
        DungeonTheme {
            particle_color: 0xb41c39ff,
            // particle_color: 0x000000ff,
            mist_color: 0xffffff09,
            dungeon_border: "castle_dungeon_nine_slice",
            floor_sprite: "floor",
            block_a_sprite: "wall",
            block_b_sprite: "firepit",
        },
        // Crypt
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
        // Pirate
        DungeonTheme {
            particle_color: 0xff9e21ff,
            mist_color: 0xffffff09,
            dungeon_border: "pirate_dungeon_nine_slice",
            floor_sprite: "wood_floor",
            block_a_sprite: "crate",
            block_b_sprite: "barrel",
        },
        // Forest
        DungeonTheme {
            particle_color: 0x6ab2c5ff,
            mist_color: 0xffffff09,
            dungeon_border: "forest_dungeon_nine_slice",
            floor_sprite: "floor_forest",
            block_a_sprite: "shrub",
            block_b_sprite: "stump",
        },
        // Ice Cave
        DungeonTheme {
            particle_color: 0x0069aaff,
            mist_color: 0xffffff09,
            dungeon_border: "ice_cave_nine_slice",
            floor_sprite: "floor_ice_cave",
            block_a_sprite: "ice_rock",
            block_b_sprite: "snowy_boulder",
        },
        // Arctic
        DungeonTheme {
            particle_color: 0x6ab2c5ff,
            mist_color: 0xffffff09,
            dungeon_border: "boreal_nine_slice",
            floor_sprite: "floor_boreal",
            block_a_sprite: "snowy_tree",
            block_b_sprite: "big_stump",
        },
    ];
    pub fn theme(&self) -> DungeonTheme {
        match self {
            Self::Castle => Self::THEMES[0],
            Self::Crypt => Self::THEMES[1],
            Self::Pirate => Self::THEMES[2],
            Self::Forest => Self::THEMES[3],
            Self::IceCave => Self::THEMES[4],
            Self::Arctic => Self::THEMES[5],
        }
    }
}
