use super::*;

pub fn fetch() -> Result<Leaderboard, std::io::Error> {
    let filepath = server::paths::global_leaderboard();
    os::client::watch_file(server::PROGRAM_ID, &filepath, &[("stream", "true")])
        .data
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Leaderboard unavailable"))
        .and_then(|file| Leaderboard::try_from_slice(&file.contents))
}
