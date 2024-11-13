use super::*;

pub fn fetch(user_id: &str) -> Result<DungeonStats, std::io::Error> {
    let filepath = server::paths::player_dungeon_stats(&user_id);
    os::client::watch_file(server::PROGRAM_ID, &filepath, &[("stream", "true")])
        .data
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "DungeonStats unavailable"))
        .and_then(|file| DungeonStats::try_from_slice(&file.contents))
}
