use super::*;

pub fn fetch(crawl_id: u32) -> Result<MultiplayerDungeon, std::io::Error> {
    let filepath = server::paths::multiplayer_dungeon(crawl_id);
    os::client::watch_file(server::PROGRAM_ID, &filepath, &[("stream", "true")])
        .data
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "MultiplayerDungeon unavailable")
        })
        .and_then(|file| MultiplayerDungeon::try_from_slice(&file.contents))
}
