use crate::vec2::Vec2;

#[derive(Debug, Copy, Clone)]
pub struct Particle {
    pub type_idx: usize,
    pub can_convert: bool,
    pub pos: Vec2,
    pub vel: Vec2,
}

impl Particle {
    pub fn new(type_idx: usize, can_convert: bool, pos: Vec2, vel: Vec2) -> Self {
        Particle { type_idx, can_convert, pos, vel }
    }
}