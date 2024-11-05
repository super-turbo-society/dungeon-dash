use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::{BTreeMap, BTreeSet};

mod client;

mod server;

mod model;
use model::*;

turbo::cfg! {r#"
    name = "Dungeon Dash"
    [settings]
    resolution = [132, 224]
    # resolution = [144, 256]
    [turbo-os]
    api-url = "https://os.turbo.computer"
    # api-url = "http://localhost:8000"
"#}

turbo::init! {
    struct LocalState {
        floor: Tween<u32>,
        turn: Tween<u32>,
        last_exec_at: usize,
        last_exec_turn: Option<u32>,
        player: Entity,
        monsters: Vec<Entity>,
        leaderboard_kind: LeaderboardKind,
        particles: Vec<Particle>,
        clouds: Vec<Cloud>,
        raindrops: Vec<Raindrop>,
        achievements_modal: Option<AchievementsModal>,
        last_crawl_achievements_modal: u32,
        show_stats_modal: bool,
    } = {
        client::ui::initialize()
    }
}

turbo::go!({
    client::ui::render();
});
