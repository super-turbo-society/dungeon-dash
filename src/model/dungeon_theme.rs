#[derive(Debug, Clone, Copy)]
pub struct DungeonTheme {
    pub particle_color: u32,
    pub mist_color: u32,
    pub dungeon_border: &'static str,
    pub floor_sprite: &'static str,
    pub block_a_sprite: &'static str,
    pub block_b_sprite: &'static str,
}
