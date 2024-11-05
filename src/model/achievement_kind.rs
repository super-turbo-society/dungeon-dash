use super::*;

// TODO: in-a-row achievements
// ("Exterminator",       "Kill all enemies for 10 floors in a row"),
// ("Untouchable",        "Don't get hit for 5 floors in a row"),
// ("Dungeon Marathon",   "Play for 3 days in a row"),

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AchievementKind {
    WelcomeToTheDungeon = 1,
    GoldGatherer = 2,
    SpaDay = 3,
    MonsterMenace = 4,
    DungeonDiver = 5,
    DungeonHobbyist = 6,
    DungeonExplorer = 7,
    DungeonConqueror = 8,
    Unbothered = 9,
    Survivalist = 10,
    GritAndGlory = 11,
    Unbreakable = 12,
    TreasureSeeker = 13,
    WealthAccumulator = 14,
    RichAdventurer = 15,
    MonsterSlayer = 16,
    MonsterVanquisher = 17,
    MonsterExterminator = 18,
    Pedestrian = 19,
    Wanderer = 20,
    Traveler = 21,
    GoblinSlayer = 22,
    OrangeMenace = 23,
    BlobBuster = 24,
    ShadeHunter = 25,
    SpiderSquasher = 26,
    Ghostbuster = 27,
    ZombieSlayer = 28,
    GoblinFodder = 29,
    Haunted = 30,
    Arachnophobia = 31,
    Blobbed = 32,
    NoviceExplorer = 33,
    SeasonedExplorer = 34,
    VeteranExplorer = 35,
    ReturningAdventurer = 36,
    DedicatedDelver = 37,
    // LearningTheRopes = 38,
    // PersistentSpirit = 39,
    SelfCare = 40,
    HealerSupreme = 41,
    // GreenGoblinHunter = 42,
    BlueBlobBuster = 43,
    SpectralBanisher = 44,
    ZombieHunter = 45,
    // Unlucky = 46,
    // Determined = 47,
    MasterOfTheDeep = 48,
    GoldHoarder = 49,
    GoldCollector = 50,
    DeadBroke = 51,
}

impl TryFrom<u32> for AchievementKind {
    type Error = ();
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::WelcomeToTheDungeon as u32 => Ok(Self::WelcomeToTheDungeon),
            x if x == Self::GoldGatherer as u32 => Ok(Self::GoldGatherer),
            x if x == Self::SpaDay as u32 => Ok(Self::SpaDay),
            x if x == Self::MonsterMenace as u32 => Ok(Self::MonsterMenace),
            x if x == Self::DungeonDiver as u32 => Ok(Self::DungeonDiver),
            x if x == Self::DungeonHobbyist as u32 => Ok(Self::DungeonHobbyist),
            x if x == Self::DungeonExplorer as u32 => Ok(Self::DungeonExplorer),
            x if x == Self::DungeonConqueror as u32 => Ok(Self::DungeonConqueror),
            x if x == Self::Unbothered as u32 => Ok(Self::Unbothered),
            x if x == Self::Survivalist as u32 => Ok(Self::Survivalist),
            x if x == Self::GritAndGlory as u32 => Ok(Self::GritAndGlory),
            x if x == Self::Unbreakable as u32 => Ok(Self::Unbreakable),
            x if x == Self::TreasureSeeker as u32 => Ok(Self::TreasureSeeker),
            x if x == Self::WealthAccumulator as u32 => Ok(Self::WealthAccumulator),
            x if x == Self::RichAdventurer as u32 => Ok(Self::RichAdventurer),
            x if x == Self::MonsterSlayer as u32 => Ok(Self::MonsterSlayer),
            x if x == Self::MonsterVanquisher as u32 => Ok(Self::MonsterVanquisher),
            x if x == Self::MonsterExterminator as u32 => Ok(Self::MonsterExterminator),
            x if x == Self::Pedestrian as u32 => Ok(Self::Pedestrian),
            x if x == Self::Wanderer as u32 => Ok(Self::Wanderer),
            x if x == Self::Traveler as u32 => Ok(Self::Traveler),
            x if x == Self::GoblinSlayer as u32 => Ok(Self::GoblinSlayer),
            x if x == Self::OrangeMenace as u32 => Ok(Self::OrangeMenace),
            x if x == Self::BlobBuster as u32 => Ok(Self::BlobBuster),
            x if x == Self::ShadeHunter as u32 => Ok(Self::ShadeHunter),
            x if x == Self::SpiderSquasher as u32 => Ok(Self::SpiderSquasher),
            x if x == Self::Ghostbuster as u32 => Ok(Self::Ghostbuster),
            x if x == Self::ZombieSlayer as u32 => Ok(Self::ZombieSlayer),
            x if x == Self::GoblinFodder as u32 => Ok(Self::GoblinFodder),
            x if x == Self::Haunted as u32 => Ok(Self::Haunted),
            x if x == Self::Arachnophobia as u32 => Ok(Self::Arachnophobia),
            x if x == Self::Blobbed as u32 => Ok(Self::Blobbed),
            x if x == Self::NoviceExplorer as u32 => Ok(Self::NoviceExplorer),
            x if x == Self::SeasonedExplorer as u32 => Ok(Self::SeasonedExplorer),
            x if x == Self::VeteranExplorer as u32 => Ok(Self::VeteranExplorer),
            x if x == Self::ReturningAdventurer as u32 => Ok(Self::ReturningAdventurer),
            x if x == Self::DedicatedDelver as u32 => Ok(Self::DedicatedDelver),
            // x if x == Self::LearningTheRopes as u32 => Ok(Self::LearningTheRopes),
            // x if x == Self::PersistentSpirit as u32 => Ok(Self::PersistentSpirit),
            x if x == Self::SelfCare as u32 => Ok(Self::SelfCare),
            x if x == Self::HealerSupreme as u32 => Ok(Self::HealerSupreme),
            // x if x == Self::GreenGoblinHunter as u32 => Ok(Self::GreenGoblinHunter),
            x if x == Self::BlueBlobBuster as u32 => Ok(Self::BlueBlobBuster),
            x if x == Self::SpectralBanisher as u32 => Ok(Self::SpectralBanisher),
            x if x == Self::ZombieHunter as u32 => Ok(Self::ZombieHunter),
            // x if x == Self::Unlucky as u32 => Ok(Self::Unlucky),
            // x if x == Self::Determined as u32 => Ok(Self::Determined),
            x if x == Self::MasterOfTheDeep as u32 => Ok(Self::MasterOfTheDeep),
            x if x == Self::GoldHoarder as u32 => Ok(Self::GoldHoarder),
            x if x == Self::GoldCollector as u32 => Ok(Self::GoldCollector),
            x if x == Self::DeadBroke as u32 => Ok(Self::DeadBroke),
            _ => {
                log!("Invalid AchievementKind u32 - {v}");
                Err(())
            }
        }
    }
}

impl AchievementKind {
    #[rustfmt::skip]
    pub const INFO: &'static [(Self, &'static str, &'static str)] = &[
        // Crawl completion (all-time)
        (Self::WelcomeToTheDungeon, "Welcome to the Dungeon", "Complete your first crawl!"),
        (Self::ReturningAdventurer, "Returning Adventurer",   "Complete 2 crawls"),
        (Self::DedicatedDelver,     "Dedicated Delver",       "Complete 5 crawls"),

        // Total floors cleared (all-time)
        (Self::NoviceExplorer,      "Novice Explorer",        "Clear 100 floors total"),
        (Self::SeasonedExplorer,    "Seasoned Explorer",      "Clear 500 floors total"),
        (Self::VeteranExplorer,     "Veteran Explorer",       "Clear 1,000 floors total"),
        (Self::MasterOfTheDeep,     "Master of the Deep",     "Clear 5,000 floors total"),

        // Floor cleared (per-crawl)
        (Self::DungeonDiver,        "Dungeon Diver",          "Reach floor 5"),
        (Self::DungeonHobbyist,     "Dungeon Hobbyist",       "Reach floor 10"),
        (Self::DungeonExplorer,     "Dungeon Explorer",       "Reach floor 15"),
        (Self::DungeonConqueror,    "Dungeon Conqueror",      "Reach floor 20"),

        // Steps moved (all-time)
        (Self::Pedestrian,          "Pedestrian",             "Take 1000 steps"),
        (Self::Wanderer,            "Wanderer",               "Take 5000 steps"),
        (Self::Traveler,            "Traveler",               "Take 10,000 steps"),

        // Gold collected (all-time)
        (Self::GoldGatherer,        "Gold Gatherer",          "Collect 1,250 gold"),
        (Self::TreasureSeeker,      "Treasure Seeker",        "Collect 2,500 gold"),
        (Self::WealthAccumulator,   "Wealth Accumulator",     "Collect 3,750 gold"),
        (Self::RichAdventurer,      "Rich Adventurer",        "Collect 5,000 gold"),

        // Gold collected (per-crawl)
        (Self::GoldCollector,       "Gold Collector",         "Collect 100 gold in one crawl"),
        (Self::GoldHoarder,         "Gold Hoarder",           "Collect 500 gold in one crawl"),
        (Self::DeadBroke,           "Dead Broke",             "Collect 0 gold in one crawl"),

        // Health recovery (all-time)
        (Self::SelfCare,            "Self Care",              "Recover 20 health"),
        (Self::SpaDay,              "Spa Day",                "Recover 100 health"),
        (Self::HealerSupreme,       "Healer Supreme",         "Recover 250 health"),

        // Health recovery (per-crawl)
        (Self::Unbothered,          "Unbothered",             "Reach floor 5 without healing"),
        (Self::Survivalist,         "Survivalist",            "Reach floor 10 without healing"),
        (Self::GritAndGlory,        "Grit & Glory",           "Reach floor 15 without healing"),
        (Self::Unbreakable,         "Unbreakable",            "Reach floor 20 without healing"),

        // Total monsters defeated (all-time)
        (Self::MonsterMenace,       "Monster Menace",         "Defeat 10 monsters"),
        (Self::MonsterSlayer,       "Monster Slayer",         "Defeat 25 monsters"),
        (Self::MonsterVanquisher,   "Monster Vanquisher",     "Defeat 50 monsters"),
        (Self::MonsterExterminator, "Monster Exterminator",   "Defeat 100 monsters"),

        // Specific monsters defeated (all-time)
        (Self::GoblinSlayer,        "Goblin Slayer",          "Defeat 50 Green Goblins"),
        (Self::OrangeMenace,        "Orange Menace",          "Defeat 50 Orange Goblins"),
        (Self::BlobBuster,          "Blob Buster",            "Defeat 50 Yellow Blobs"),
        (Self::ShadeHunter,         "Shade Hunter",           "Defeat 50 Shades"),
        (Self::SpiderSquasher,      "Spider Squasher",        "Defeat 50 Spiders"),
        (Self::Ghostbuster,         "Ghostbuster",            "Defeat 50 Ghosts"),
        (Self::ZombieSlayer,        "Zombie Slayer",          "Defeat 50 Zombies"),
        (Self::BlueBlobBuster,      "Blue Blob Buster",       "Defeat 50 Blue Blobs"),
        (Self::ZombieHunter,        "Zombie Hunter",          "Defeat 50 Zombies"),
        (Self::SpectralBanisher,    "Spectral Banisher",      "Defeat 5 Spectral Ghost"),

        // Defeats by specific monsters (all-time)
        (Self::GoblinFodder,        "Goblin Fodder",          "Defeated by Green Goblin 5 times"),
        (Self::Haunted,             "Haunted",                "Defeated by Ghost 5 times"),
        (Self::Arachnophobia,       "Arachnophobia",          "Defeated by Spider 5 times"),
        (Self::Blobbed,             "Blobbed",                "Defeated by Red Blob 5 times"),
    ];

    pub fn info(self) -> (&'static str, &'static str) {
        let (_, name, description) = Self::INFO.iter().find(|a| a.0 == self).unwrap();
        (name, description)
    }

    #[rustfmt::skip]
    pub fn test(&self, crawl_stats: &DungeonStats, total_stats: &DungeonStats, did_crawl_end: bool) -> bool {
        match self {
            // Crawl completion (all-time)
            Self::WelcomeToTheDungeon => total_stats.get(DungeonStatKind::CrawlsCompleted) >= 1,
            Self::ReturningAdventurer => total_stats.get(DungeonStatKind::CrawlsCompleted) >= 2,
            Self::DedicatedDelver => total_stats.get(DungeonStatKind::CrawlsCompleted) >= 5,

            // Total floors cleared (all-time)
            Self::NoviceExplorer => total_stats.get(DungeonStatKind::FloorsCleared) >= 100,
            Self::SeasonedExplorer => total_stats.get(DungeonStatKind::FloorsCleared) >= 500,
            Self::VeteranExplorer => total_stats.get(DungeonStatKind::FloorsCleared) >= 1000,
            Self::MasterOfTheDeep => total_stats.get(DungeonStatKind::FloorsCleared) >= 5000,

            // Floors cleared (per-crawl)
            Self::DungeonDiver => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 5,
            Self::DungeonHobbyist => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 10,
            Self::DungeonExplorer => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 15,
            Self::DungeonConqueror => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 20,

            // Steps moved (all-time)
            Self::Pedestrian => total_stats.get(DungeonStatKind::StepsMoved) >= 1000,
            Self::Wanderer => total_stats.get(DungeonStatKind::StepsMoved) >= 5000,
            Self::Traveler => total_stats.get(DungeonStatKind::StepsMoved) >= 10000,

            // Gold collected (all-time)
            Self::GoldGatherer => total_stats.get(DungeonStatKind::GoldCollected) >= 1250,
            Self::TreasureSeeker => total_stats.get(DungeonStatKind::GoldCollected) >= 2500,
            Self::WealthAccumulator => total_stats.get(DungeonStatKind::GoldCollected) >= 3750,
            Self::RichAdventurer => total_stats.get(DungeonStatKind::GoldCollected) >= 5000,

            // Gold collected (per-crawl)
            Self::GoldCollector => crawl_stats.get(DungeonStatKind::GoldCollected) >= 100,
            Self::GoldHoarder => crawl_stats.get(DungeonStatKind::GoldCollected) >= 500,
            Self::DeadBroke => crawl_stats.get(DungeonStatKind::GoldCollected) == 0 && did_crawl_end,

            // Health recovery (all-time)
            Self::SelfCare => total_stats.get(DungeonStatKind::HealthRecovered) >= 20,
            Self::SpaDay => total_stats.get(DungeonStatKind::HealthRecovered) >= 100,
            Self::HealerSupreme => total_stats.get(DungeonStatKind::HealthRecovered) >= 250,

            // Health recovery (per-crawl)
            Self::Unbothered => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 5 && crawl_stats.get(DungeonStatKind::HealthRecovered) == 0,
            Self::Survivalist => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 10 && crawl_stats.get(DungeonStatKind::HealthRecovered) == 0,
            Self::GritAndGlory => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 15 && crawl_stats.get(DungeonStatKind::HealthRecovered) == 0,
            Self::Unbreakable => crawl_stats.get(DungeonStatKind::FloorsCleared) >= 20 && crawl_stats.get(DungeonStatKind::HealthRecovered) == 0,

            // Total monsters defeated (all-time)
            Self::MonsterMenace => total_stats.total_monsters_defeated() >= 10,
            Self::MonsterSlayer => total_stats.total_monsters_defeated() >= 25,
            Self::MonsterVanquisher => total_stats.total_monsters_defeated() >= 50,
            Self::MonsterExterminator => total_stats.total_monsters_defeated() >= 100,

            // Specific monsters defeated (all-time)
            Self::GoblinSlayer => total_stats.monster_kills(MonsterKind::GreenGoblin) >= 50,
            Self::OrangeMenace => total_stats.monster_kills(MonsterKind::OrangeGoblin) >= 50,
            Self::BlobBuster => total_stats.monster_kills(MonsterKind::YellowBlob) >= 50,
            Self::ShadeHunter => total_stats.monster_kills(MonsterKind::Shade) >= 50,
            Self::SpiderSquasher => total_stats.monster_kills(MonsterKind::Spider) >= 50,
            Self::Ghostbuster => total_stats.monster_kills(MonsterKind::Ghost) >= 50,
            Self::ZombieSlayer => total_stats.monster_kills(MonsterKind::Zombie) >= 50,
            Self::BlueBlobBuster => total_stats.monster_kills(MonsterKind::BlueBlob) >= 50,
            Self::ZombieHunter => total_stats.monster_kills(MonsterKind::Zombie) >= 50,
            Self::SpectralBanisher => total_stats.monster_kills(MonsterKind::SpectralGhost) >= 5,

            // Defeats by specific monsters (all-time)
            Self::GoblinFodder => total_stats.deaths_by_monster(MonsterKind::GreenGoblin) >= 5,
            Self::Haunted => total_stats.deaths_by_monster(MonsterKind::Ghost) >= 5,
            Self::Arachnophobia => total_stats.deaths_by_monster(MonsterKind::Spider) >= 5,
            Self::Blobbed => total_stats.deaths_by_monster(MonsterKind::RedBlob) >= 5,
        }
    }
}
