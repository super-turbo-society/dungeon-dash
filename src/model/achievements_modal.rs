use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct AchievementsModal {
    pub y: Tween<i32>,
    pub current: usize,
    pub kinds: Vec<AchievementKind>,
    pub confetti: Vec<Confetti>,
}
impl AchievementsModal {
    pub const TRANSITION_DUR: usize = 48;
    pub fn new(kinds: &[AchievementKind]) -> Self {
        let [w, h] = canvas_size!();
        Self {
            y: Tween::new(-(h as i32))
                .duration(Self::TRANSITION_DUR)
                .ease(Easing::EaseInOutQuad),
            current: 0,
            kinds: kinds.to_vec(),
            confetti: {
                let mut confetti = vec![];
                for _ in 0..50 {
                    confetti.push(Confetti {
                        x: (rand() % w) as f32,
                        y: (rand() % h) as f32,
                        radius: (rand() % 5 + 2) as f32,
                        color: rand() % 0xFFFFFF11 | 0xFF00ff88,
                        vy: (rand() % 2 + 1) as f32,
                    });
                }
                confetti
            },
        }
    }
}
