use super::*;

pub fn fetch() -> Result<BTreeMap<String, MultiplayerDungeonLobby>, std::io::Error> {
    let filepath = server::paths::multiplayer_dungeon_list();
    let res = os::client::watch_file(server::PROGRAM_ID, &filepath, &[("stream", "true")]);
    // Default empty list if the file does not yet exist
    if !res.loading && res.error.is_none() && res.data.is_none() {
        return Ok(BTreeMap::new());
    }
    // Otherwise, parse the response data
    res.data
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "lobby list unavailable"))
        .and_then(|file| {
            <BTreeMap<String, MultiplayerDungeonLobby>>::try_from_slice(&file.contents)
        })
}
