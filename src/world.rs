use std::collections::HashMap;

use crate::block::Block;
use crate::chunk::{CHUNK_SIZE, Chunk, ChunkPos, chunk_coord, in_chunk};

pub struct World {
    age_ticks: u64,
    chunks: HashMap<ChunkPos, Chunk<Block>>,
}

impl World {
    pub fn new() -> Self {
        let mut w = Self {
            age_ticks: 0,
            chunks: HashMap::new(),
        };

        // Startbereich: Bodenplatte + kleine Wand wie vorher (nur größer, chunk-safe)
        w.ensure_spawn_area();
        w
    }

    pub fn size(&self) -> i32 {
        // Alte API: Mini-Welt war 16. Für jetzt als "default".
        // Kann später raus, wenn Game keine size mehr braucht.
        16
    }

    pub fn get_block_opt(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        // Alte API: Out-of-bounds = None
        // Neue Chunk-Welt ist "unbounded", also immer Some(...)
        Some(self.get_block(x, y, z))
    }

    fn mark_dirty(&mut self, cp: ChunkPos) {
        if let Some(ch) = self.chunks.get_mut(&cp) {
            ch.dirty = true;
        }
    }

    /// Gibt zurück, ob der Chunk 'dirty' war, und setzt dirty=false.
    pub fn take_chunk_dirty(&mut self, cp: ChunkPos) -> bool {
        if let Some(ch) = self.chunks.get_mut(&cp) {
            let was = ch.dirty;
            ch.dirty = false;
            was
        } else {
            false
        }
    }

    pub fn tick(&mut self) {
        self.age_ticks += 1;
    }

    pub fn age(&self) -> u64 {
        self.age_ticks
    }

    /// Optional: Debug/Info – Anzahl geladener Chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn chunk_positions(&self) -> Vec<ChunkPos> {
        self.chunks.keys().copied().collect()
    }

    fn get_or_create_chunk(&mut self, pos: ChunkPos) -> &mut Chunk<Block> {
        self.chunks.entry(pos).or_insert_with(|| Chunk::new(pos))
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Block {
        let cp = ChunkPos {
            cx: chunk_coord(x),
            cy: chunk_coord(y),
            cz: chunk_coord(z),
        };

        let lx = in_chunk(x);
        let ly = in_chunk(y);
        let lz = in_chunk(z);

        match self.chunks.get(&cp) {
            Some(ch) => ch.get_local(lx, ly, lz),
            None => Block::Air,
        }
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, b: Block) -> bool {
        let cx = chunk_coord(x);
        let cy = chunk_coord(y);
        let cz = chunk_coord(z);

        let lx = in_chunk(x);
        let ly = in_chunk(y);
        let lz = in_chunk(z);

        let cp = ChunkPos { cx, cy, cz };

        // Chunk anlegen + setzen (setzt dirty ohnehin)
        {
            let ch = self.get_or_create_chunk(cp);
            ch.set_local(lx, ly, lz, b);
        }

        // Wenn an Chunk-Kante geändert → Nachbarn dirty
        if lx == 0 {
            self.mark_dirty(ChunkPos { cx: cx - 1, cy, cz });
        } else if lx == CHUNK_SIZE - 1 {
            self.mark_dirty(ChunkPos { cx: cx + 1, cy, cz });
        }

        if ly == 0 {
            self.mark_dirty(ChunkPos { cx, cy: cy - 1, cz });
        } else if ly == CHUNK_SIZE - 1 {
            self.mark_dirty(ChunkPos { cx, cy: cy + 1, cz });
        }

        if lz == 0 {
            self.mark_dirty(ChunkPos { cx, cy, cz: cz - 1 });
        } else if lz == CHUNK_SIZE - 1 {
            self.mark_dirty(ChunkPos { cx, cy, cz: cz + 1 });
        }

        true
    }

    pub fn break_block(&mut self, x: i32, y: i32, z: i32) -> bool {
        self.set_block(x, y, z, Block::Air)
    }

    pub fn place_block(&mut self, x: i32, y: i32, z: i32, b: Block) -> bool {
        self.set_block(x, y, z, b)
    }

    pub fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        self.get_block(x, y, z) != Block::Air
    }

    pub fn ensure_spawn_area(&mut self) {
        // Ein Feld von 64x64 auf y=0 als Dirt
        for x in 0..64 {
            for z in 0..64 {
                self.set_block(x, 0, z, Block::Dirt);
            }
        }

        // Test-Wand wie vorher (z=8, x=3..5, y=1..3)
        for y in 1..=3 {
            for x in 3..=5 {
                self.set_block(x, y, 8, Block::Stone);
            }
        }

        // Optional: ein paar Chunks "anlegen", damit HashMap schon gefüllt ist
        // (nicht notwendig, aber manchmal hilfreich beim Debuggen)
        let _ = CHUNK_SIZE; // nur, damit Import nicht als "unused" gilt, falls du’s nicht nutzt
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
        let b0 = self.get_block(vx, vy, vz);
        if b0 != Block::Air {
            return Some((vx, vy, vz, b0, (0, 0, 0)));
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

            let b = self.get_block(vx, vy, vz);
            if b != Block::Air {
                return Some((vx, vy, vz, b, hit_normal));
            }
        }

        None
    }
}
