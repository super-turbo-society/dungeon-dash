use super::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct Entity {
    pub x: Tween<i32>,
    pub y: Tween<i32>,
    pub offset_x: Tween<i32>,
    pub offset_y: Tween<i32>,
}
impl Entity {
    pub fn is_idle(&mut self) -> bool {
        let is_x_done = self.x.done();
        let is_y_done = self.y.done();
        let is_offset_x_done = self.offset_x.done();
        let is_offset_y_done = self.offset_y.done();
        is_x_done && is_y_done && is_offset_x_done && is_offset_y_done
    }
}