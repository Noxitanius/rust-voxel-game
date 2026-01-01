#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Block {
    Air,
    Dirt,
    Stone,
}

impl Default for Block {
    fn default() -> Self {
        Block::Air
    }
}
