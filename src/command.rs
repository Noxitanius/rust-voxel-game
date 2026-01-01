use crate::block::Block;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Break { x: i32, y: i32, z: i32 },
    Place { x: i32, y: i32, z: i32, block: Block },
}