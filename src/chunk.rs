use std::hash::{Hash, Hasher};

pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_VOL: usize = (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize);

/// Chunk-Koordinate im Chunk-Raster (nicht in Block-Koordinaten!)
#[derive(Debug, Clone, Copy, Eq)]
pub struct ChunkPos {
    pub cx: i32,
    pub cy: i32,
    pub cz: i32,
}

impl ChunkPos {
    pub fn new(cx: i32, cy: i32, cz: i32) -> Self {
        Self { cx, cy, cz }
    }
}

impl PartialEq for ChunkPos {
    fn eq(&self, other: &Self) -> bool {
        self.cx == other.cx && self.cy == other.cy && self.cz == other.cz
    }
}

impl Hash for ChunkPos {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cx.hash(state);
        self.cy.hash(state);
        self.cz.hash(state);
    }
}

/// Lokale Block-Koordinate im Chunk: 0..15
#[inline]
pub fn in_chunk(v: i32) -> i32 {
    // v mod 16, aber immer positiv
    v.rem_euclid(CHUNK_SIZE)
}

/// Chunk-Koordinate aus Block-Koordinate
#[inline]
pub fn chunk_coord(v: i32) -> i32 {
    // floor-div für negative Werte korrekt
    v.div_euclid(CHUNK_SIZE)
}

/// Linearisierter Index [0..4095] aus lokalen Koordinaten [0..15]
#[inline]
pub fn idx(lx: i32, ly: i32, lz: i32) -> usize {
    // Layout: X läuft am schnellsten, dann Z, dann Y
    // index = x + z*16 + y*16*16
    (lx as usize)
        + (lz as usize) * (CHUNK_SIZE as usize)
        + (ly as usize) * (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize)
}

#[derive(Debug, Clone)]
pub struct Chunk<B: Copy + Default> {
    pub pos: ChunkPos,
    blocks: Vec<B>, // Länge: 4096
    pub dirty: bool,
}

impl<B: Copy + Default> Chunk<B> {
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            blocks: vec![B::default(); CHUNK_VOL],
            dirty: true,
        }
    }

    #[inline]
    pub fn get_local(&self, lx: i32, ly: i32, lz: i32) -> B {
        self.blocks[idx(lx, ly, lz)]
    }

    #[inline]
    pub fn set_local(&mut self, lx: i32, ly: i32, lz: i32, b: B) {
        let i = idx(lx, ly, lz);
        self.blocks[i] = b;
        self.dirty = true;
    }
}
