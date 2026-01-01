use crate::block::Block;
use crate::chunk::{ChunkPos, CHUNK_SIZE};
use crate::mesh::Vertex;
use crate::world::World;

fn block_color(b: Block) -> [f32; 3] {
    match b {
        Block::Air => [0.0, 0.0, 0.0],      // wird nicht gerendert
        Block::Dirt => [0.55, 0.40, 0.20],
        Block::Stone => [0.60, 0.60, 0.60],
    }
}

#[inline]
fn is_air(b: Block) -> bool {
    b == Block::Air
}

/// Baut das Mesh für genau einen Chunk.
/// Gibt Vertices + Indices zurück (Indices sind u32).
pub fn mesh_chunk(world: &World, cp: ChunkPos) -> (Vec<Vertex>, Vec<u32>) {
    let mut verts: Vec<Vertex> = Vec::new();
    let mut inds: Vec<u32> = Vec::new();

    // Chunk-Origin in Block-Koordinaten
    let ox = cp.cx * CHUNK_SIZE;
    let oy = cp.cy * CHUNK_SIZE;
    let oz = cp.cz * CHUNK_SIZE;

    for ly in 0..CHUNK_SIZE {
        for lz in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let x = ox + lx;
                let y = oy + ly;
                let z = oz + lz;

                let b = world.get_block(x, y, z);
                if is_air(b) {
                    continue;
                }

                let col = block_color(b);

                // Für jede Seite: wenn Nachbar Air -> Face hinzufügen
                // +X
                if is_air(world.get_block(x + 1, y, z)) {
                    push_face(&mut verts, &mut inds, col,
                        [x as f32 + 1.0, y as f32, z as f32],
                        [x as f32 + 1.0, y as f32 + 1.0, z as f32],
                        [x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0],
                        [x as f32 + 1.0, y as f32, z as f32 + 1.0],
                    );
                }
                // -X
                if is_air(world.get_block(x - 1, y, z)) {
                    push_face(&mut verts, &mut inds, col,
                        [x as f32, y as f32, z as f32 + 1.0],
                        [x as f32, y as f32 + 1.0, z as f32 + 1.0],
                        [x as f32, y as f32 + 1.0, z as f32],
                        [x as f32, y as f32, z as f32],
                    );
                }
                // +Y (top)
                if is_air(world.get_block(x, y + 1, z)) {
                    push_face(&mut verts, &mut inds, col,
                        [x as f32, y as f32 + 1.0, z as f32],
                        [x as f32, y as f32 + 1.0, z as f32 + 1.0],
                        [x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0],
                        [x as f32 + 1.0, y as f32 + 1.0, z as f32],
                    );
                }
                // -Y (bottom)
                if is_air(world.get_block(x, y - 1, z)) {
                    push_face(&mut verts, &mut inds, col,
                        [x as f32 + 1.0, y as f32, z as f32],
                        [x as f32 + 1.0, y as f32, z as f32 + 1.0],
                        [x as f32, y as f32, z as f32 + 1.0],
                        [x as f32, y as f32, z as f32],
                    );
                }
                // +Z
                if is_air(world.get_block(x, y, z + 1)) {
                    push_face(&mut verts, &mut inds, col,
                        [x as f32 + 1.0, y as f32, z as f32 + 1.0],
                        [x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0],
                        [x as f32, y as f32 + 1.0, z as f32 + 1.0],
                        [x as f32, y as f32, z as f32 + 1.0],
                    );
                }
                // -Z
                if is_air(world.get_block(x, y, z - 1)) {
                    push_face(&mut verts, &mut inds, col,
                        [x as f32, y as f32, z as f32],
                        [x as f32, y as f32 + 1.0, z as f32],
                        [x as f32 + 1.0, y as f32 + 1.0, z as f32],
                        [x as f32 + 1.0, y as f32, z as f32],
                    );
                }
            }
        }
    }

    (verts, inds)
}

#[inline]
fn push_face(verts: &mut Vec<Vertex>, inds: &mut Vec<u32>, color: [f32; 3],
             p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3]) {
    let base = verts.len() as u32;

    verts.push(Vertex { pos: p0, color });
    verts.push(Vertex { pos: p1, color });
    verts.push(Vertex { pos: p2, color });
    verts.push(Vertex { pos: p3, color });

    // zwei Dreiecke (0,1,2) und (0,2,3)
    inds.extend_from_slice(&[
        base, base + 1, base + 2,
        base, base + 2, base + 3,
    ]);
}
