use super::*;

pub fn fetch(user_id: &str) -> Result<Dungeon, std::io::Error> {
    let filepath = server::paths::player_dungeon(&user_id);
    os::client::watch_file(server::PROGRAM_ID, &filepath)
        .data
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Dungeon unavailable"))
        .and_then(|file| Dungeon::try_from_slice(&file.contents))
}
