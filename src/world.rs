use crate::block::Block;

pub struct World {
    age_ticks: u64,

    // Mini-Welt: 16×16×16
    size: i32,
    blocks: Vec<Block>,
}

impl World {
    pub fn new() -> Self {
        let size = 16;
        let total = (size * size * size) as usize;

        let mut blocks = vec![Block::Air; total];

        // Bodenplatte aus Dirt bei y=0
        for z in 0..size {
            for x in 0..size {
                let idx = Self::index(size, x, 0, z);
                blocks[idx] = Block::Dirt;
            }
        }

        // Test: kleine Wand aus Stone bei z=8, x=3..5, y=1..3
        for y in 1..=3 {
            for x in 3..=5 {
                let idx = Self::index(size, x, y, 8);
                blocks[idx] = Block::Stone;
            }
        }

        Self {
            age_ticks: 0,
            size,
            blocks,
        }
    }

    pub fn tick(&mut self) {
        self.age_ticks += 1;
    }

    pub fn age(&self) -> u64 {
        self.age_ticks
    }

    pub fn size(&self) -> i32 {
        self.size
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        if !Self::in_bounds(self.size, x, y, z) {
            return None;
        }
        let idx = Self::index(self.size, x, y, z);
        Some(self.blocks[idx])
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, b: Block) -> bool {
        if !Self::in_bounds(self.size, x, y, z) {
            return false;
        }
        let idx = Self::index(self.size, x, y, z);
        self.blocks[idx] = b;
        true
    }

    pub fn break_block(&mut self, x: i32, y: i32, z: i32) -> bool {
        self.set_block(x, y, z, Block::Air)
    }

    pub fn place_block(&mut self, x: i32, y: i32, z: i32, b: Block) -> bool {
        self.set_block(x, y, z, b)
    }

    pub fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        match self.get_block(x, y, z) {
            Some(Block::Air) | None => false,
            Some(_) => true, // Dirt, Stone, ...
        }
    }

    fn in_bounds(size: i32, x: i32, y: i32, z: i32) -> bool {
        x >= 0 && x < size && y >= 0 && y < size && z >= 0 && z < size
    }

    fn index(size: i32, x: i32, y: i32, z: i32) -> usize {
        // x + size*(y + size*z)
        (x + size * (y + size * z)) as usize
    }

    pub fn raycast_first_solid(
        &self,
        start_x: f32,
        start_y: f32,
        start_z: f32,
        dir_x: f32,
        dir_y: f32,
        dir_z: f32,
        max_dist: f32,
    ) -> Option<(i32, i32, i32, Block, (i32, i32, i32))> {
        if dir_x == 0.0 && dir_y == 0.0 && dir_z == 0.0 {
            return None;
        }

        let mut vx = start_x.floor() as i32;
        let mut vy = start_y.floor() as i32;
        let mut vz = start_z.floor() as i32;

        let step_x = if dir_x > 0.0 {
            1
        } else if dir_x < 0.0 {
            -1
        } else {
            0
        };
        let step_y = if dir_y > 0.0 {
            1
        } else if dir_y < 0.0 {
            -1
        } else {
            0
        };
        let step_z = if dir_z > 0.0 {
            1
        } else if dir_z < 0.0 {
            -1
        } else {
            0
        };

        let inv_x = if dir_x != 0.0 {
            1.0 / dir_x.abs()
        } else {
            f32::INFINITY
        };
        let inv_y = if dir_y != 0.0 {
            1.0 / dir_y.abs()
        } else {
            f32::INFINITY
        };
        let inv_z = if dir_z != 0.0 {
            1.0 / dir_z.abs()
        } else {
            f32::INFINITY
        };

        let next_boundary_x = if step_x > 0 {
            (vx + 1) as f32
        } else {
            vx as f32
        };
        let next_boundary_y = if step_y > 0 {
            (vy + 1) as f32
        } else {
            vy as f32
        };
        let next_boundary_z = if step_z > 0 {
            (vz + 1) as f32
        } else {
            vz as f32
        };

        let mut t_max_x = if dir_x != 0.0 {
            (next_boundary_x - start_x).abs() * inv_x
        } else {
            f32::INFINITY
        };
        let mut t_max_y = if dir_y != 0.0 {
            (next_boundary_y - start_y).abs() * inv_y
        } else {
            f32::INFINITY
        };
        let mut t_max_z = if dir_z != 0.0 {
            (next_boundary_z - start_z).abs() * inv_z
        } else {
            f32::INFINITY
        };

        let t_delta_x = inv_x;
        let t_delta_y = inv_y;
        let t_delta_z = inv_z;

        let mut t = 0.0;
        let mut hit_normal = (0, 0, 0);

        // Start-Block prüfen
        if let Some(b) = self.get_block(vx, vy, vz) {
            if b != Block::Air {
                return Some((vx, vy, vz, b, (0, 0, 0)));
            }
        } else {
            return None;
        }

        while t <= max_dist {
            if t_max_x < t_max_y && t_max_x < t_max_z {
                vx += step_x;
                t = t_max_x;
                t_max_x += t_delta_x;
                hit_normal = (-step_x, 0, 0);
            } else if t_max_y < t_max_z {
                vy += step_y;
                t = t_max_y;
                t_max_y += t_delta_y;
                hit_normal = (0, -step_y, 0);
            } else {
                vz += step_z;
                t = t_max_z;
                t_max_z += t_delta_z;
                hit_normal = (0, 0, -step_z);
            }

            let b = match self.get_block(vx, vy, vz) {
                Some(b) => b,
                None => return None,
            };

            if b != Block::Air {
                return Some((vx, vy, vz, b, hit_normal));
            }
        }

        None
    }
}
