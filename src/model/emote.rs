use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum Emote {
    Love,
    Anger,
    Sob,
    Thinking,
}