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

/// Greedy-Meshing ffr einen Chunk: kombiniert benachbarte gleichfarbige Quads
/// auf jeder Achse und reduziert so die Vertex-/Index-Anzahl.
pub fn mesh_chunk(world: &World, cp: ChunkPos) -> (Vec<Vertex>, Vec<u32>) {
    let mut verts: Vec<Vertex> = Vec::new();
    let mut inds: Vec<u32> = Vec::new();

    let ox = cp.cx * CHUNK_SIZE;
    let oy = cp.cy * CHUNK_SIZE;
    let oz = cp.cz * CHUNK_SIZE;

    let size = CHUNK_SIZE as usize;
    let mut mask: Vec<Option<(Block, bool)>> = vec![None; size * size];

    // Achse 0 = X, 1 = Y, 2 = Z
    for axis in 0..3 {
        for d in 0..=CHUNK_SIZE {
            // Maske f11llen: Unterschiede zwischen Slice d-1 und d
            for j in 0..CHUNK_SIZE {
                for i in 0..CHUNK_SIZE {
                    let idx = (j as usize) * size + (i as usize);

                    let (a, b) = match axis {
                        0 => {
                            // X-Achse: i -> Z, j -> Y
                            let y = j;
                            let z = i;
                            let a = if d > 0 {
                                world.get_block(ox + (d - 1), oy + y, oz + z)
                            } else {
                                Block::Air
                            };
                            let b = if d < CHUNK_SIZE {
                                world.get_block(ox + d, oy + y, oz + z)
                            } else {
                                Block::Air
                            };
                            (a, b)
                        }
                        1 => {
                            // Y-Achse: i -> X, j -> Z
                            let x = i;
                            let z = j;
                            let a = if d > 0 {
                                world.get_block(ox + x, oy + (d - 1), oz + z)
                            } else {
                                Block::Air
                            };
                            let b = if d < CHUNK_SIZE {
                                world.get_block(ox + x, oy + d, oz + z)
                            } else {
                                Block::Air
                            };
                            (a, b)
                        }
                        _ => {
                            // Z-Achse: i -> X, j -> Y
                            let x = i;
                            let y = j;
                            let a = if d > 0 {
                                world.get_block(ox + x, oy + y, oz + (d - 1))
                            } else {
                                Block::Air
                            };
                            let b = if d < CHUNK_SIZE {
                                world.get_block(ox + x, oy + y, oz + d)
                            } else {
                                Block::Air
                            };
                            (a, b)
                        }
                    };

                    if !is_air(a) || !is_air(b) {
                        if !is_air(a) && is_air(b) {
                            // Face zeigt in -Axis Richtung
                            mask[idx] = Some((a, false));
                        } else if is_air(a) && !is_air(b) {
                            // Face zeigt in +Axis Richtung
                            mask[idx] = Some((b, true));
                        } else {
                            mask[idx] = None;
                        }
                    } else {
                        mask[idx] = None;
                    }
                }
            }

            // Greedy zusammenfassen
            let mut idx = 0;
            while idx < mask.len() {
                if let Some((block, pos_side)) = mask[idx] {
                    let mut w = 1usize;
                    while (idx % size) + w < size && mask[idx + w] == Some((block, pos_side)) {
                        w += 1;
                    }

                    let mut h = 1usize;
                    'outer: while (idx / size) + h < size {
                        for k in 0..w {
                            if mask[idx + k + h * size] != Some((block, pos_side)) {
                                break 'outer;
                            }
                        }
                        h += 1;
                    }

                    let i0 = (idx % size) as i32;
                    let j0 = (idx / size) as i32;
                    push_quad(
                        axis,
                        pos_side,
                        d,
                        i0,
                        j0,
                        w as i32,
                        h as i32,
                        block_color(block),
                        ox,
                        oy,
                        oz,
                        &mut verts,
                        &mut inds,
                    );

                    // Maske leeren
                    for dy in 0..h {
                        for dx in 0..w {
                            mask[idx + dx + dy * size] = None;
                        }
                    }
                }
                idx += 1;
            }
        }
    }

    (verts, inds)
}

fn push_quad(
    axis: i32,
    pos_side: bool,
    d: i32,
    i0: i32,
    j0: i32,
    w: i32,
    h: i32,
    color: [f32; 3],
    ox: i32,
    oy: i32,
    oz: i32,
    verts: &mut Vec<Vertex>,
    inds: &mut Vec<u32>,
) {
    // Baut einen Quad mit CCW-Winding zur Außen-Normale.
    let (p0, p1, p2, p3) = match (axis, pos_side) {
        // X-Achse
        (0, true) => {
            let x = (ox + d) as f32;
            let y0 = (oy + j0) as f32;
            let y1 = (oy + j0 + h) as f32;
            let z0 = (oz + i0) as f32;
            let z1 = (oz + i0 + w) as f32;
            (
                [x, y0, z0],
                [x, y1, z0],
                [x, y1, z1],
                [x, y0, z1],
            )
        }
        (0, false) => {
            let x = (ox + d - 1) as f32;
            let y0 = (oy + j0) as f32;
            let y1 = (oy + j0 + h) as f32;
            let z0 = (oz + i0 + w) as f32;
            let z1 = (oz + i0) as f32;
            (
                [x, y0, z0],
                [x, y1, z0],
                [x, y1, z1],
                [x, y0, z1],
            )
        }
        // Y-Achse
        (1, true) => {
            let y = (oy + d) as f32;
            let x0 = (ox + i0) as f32;
            let x1 = (ox + i0 + w) as f32;
            let z0 = (oz + j0) as f32;
            let z1 = (oz + j0 + h) as f32;
            (
                [x0, y, z0],
                [x0, y, z1],
                [x1, y, z1],
                [x1, y, z0],
            )
        }
        (1, false) => {
            let y = (oy + d - 1) as f32;
            let x0 = (ox + i0) as f32;
            let x1 = (ox + i0 + w) as f32;
            let z0 = (oz + j0) as f32;
            let z1 = (oz + j0 + h) as f32;
            (
                [x0, y, z0],
                [x0, y, z1],
                [x1, y, z1],
                [x1, y, z0],
            )
        }
        // Z-Achse
        (2, true) => {
            let z = (oz + d) as f32;
            let x0 = (ox + i0) as f32;
            let x1 = (ox + i0 + w) as f32;
            let y0 = (oy + j0) as f32;
            let y1 = (oy + j0 + h) as f32;
            (
                [x0, y0, z],
                [x0, y1, z],
                [x1, y1, z],
                [x1, y0, z],
            )
        }
        (2, false) => {
            let z = (oz + d - 1) as f32;
            let x0 = (ox + i0) as f32;
            let x1 = (ox + i0 + w) as f32;
            let y0 = (oy + j0) as f32;
            let y1 = (oy + j0 + h) as f32;
            (
                [x0, y0, z],
                [x0, y1, z],
                [x1, y1, z],
                [x1, y0, z],
            )
        }
        _ => unreachable!(),
    };

    let base = verts.len() as u32;
    verts.push(Vertex { pos: p0, color });
    verts.push(Vertex { pos: p1, color });
    verts.push(Vertex { pos: p2, color });
    verts.push(Vertex { pos: p3, color });

    inds.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
