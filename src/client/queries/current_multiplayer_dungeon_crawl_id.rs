use super::*;

pub fn fetch(user_id: &str) -> Result<u32, std::io::Error> {
    let filepath = server::paths::player_multiplayer_dungeon_manifest(&user_id);
    os::client::watch_file(server::PROGRAM_ID, &filepath, &[("stream", "true")])
        .data
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "crawl_id unavailable"))
        .and_then(|file| <u32>::try_from_slice(&file.contents))
}
