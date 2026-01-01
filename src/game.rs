use crate::block::Block;
use crate::chunk::{chunk_coord, ChunkPos, CHUNK_SIZE};
use crate::command::Command;
use crate::input::InputState;
use crate::mesh::Vertex;
use crate::player::Player;
use crate::voxel_mesher::mesh_chunk;
use crate::world::World;
use glam::Vec3;
use std::collections::HashMap;

const CAMERA_FOV_Y: f32 = 45.0_f32.to_radians();
const CAMERA_FAR: f32 = 200.0;

pub struct Game {
    tick: u64,
    world: World,
    player: Player,
    commands: Vec<Command>,
    chunk_mesh_cache: HashMap<ChunkPos, (Vec<Vertex>, Vec<u32>)>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            tick: 0,
            world: World::new(),
            player: Player::new(),
            commands: Vec::new(),
            chunk_mesh_cache: HashMap::new(),
        }
    }

    pub fn look_delta(&mut self, dx: f32, dy: f32) {
        // native Mausbewegung (kein invert)
        self.player.add_look(dx, dy);
    }

    pub fn apply_movement(&mut self, input: InputState) {
        // 20 TPS => dt = 0.05s
        let dt = 0.05_f32;
        let speed = 4.0_f32; // Blöcke pro Sekunde (gefühlvoll, anpassbar)
        let step = speed * dt;

        // Vorwärtsrichtung nur in XZ (ohne hoch/runter)
        let (dx, _dy, dz) = self.player.dir();

        // Normalisieren in XZ
        let mut fwd_x = dx;
        let mut fwd_z = dz;
        let len = (fwd_x * fwd_x + fwd_z * fwd_z).sqrt();
        if len > 0.0001 {
            fwd_x /= len;
            fwd_z /= len;
        }

        // Rechtsvektor (90° gedreht)
        let right_x = fwd_z;
        let right_z = -fwd_x;

        let mut mx = 0.0_f32;
        let mut mz = 0.0_f32;

        if input.move_fwd {
            mx += fwd_x;
            mz += fwd_z;
        }
        if input.move_back {
            mx -= fwd_x;
            mz -= fwd_z;
        }
        if input.move_right {
            mx += right_x;
            mz += right_z;
        }
        if input.move_left {
            mx -= right_x;
            mz -= right_z;
        }

        // Diagonal nicht schneller
        let mlen = (mx * mx + mz * mz).sqrt();
        if mlen > 0.0001 {
            mx /= mlen;
            mz /= mlen;

            let target_x = self.player.x + mx * step;
            let target_z = self.player.z + mz * step;

            // erst X bewegen
            if !self.collides_at(target_x, self.player.y, self.player.z) {
                self.player.x = target_x;
            } else {
                // Step-up versuchen (nur wenn wir grundsätzlich "laufen")
                let _ = self.try_step_up(target_x, self.player.z);
            }

            // dann Z bewegen
            if !self.collides_at(self.player.x, self.player.y, target_z) {
                self.player.z = target_z;
            } else {
                let _ = self.try_step_up(self.player.x, target_z);
            }
        }
    }

    pub fn apply_vertical_physics(&mut self, input: InputState) {
        let dt = 0.05_f32; // 20 TPS
        let gravity = 18.0_f32; // Blöcke/s^2
        let jump_v = 7.0_f32; // Sprungimpuls

        // Jump (one-shot)
        if input.jump && self.player.on_ground {
            self.player.vy = jump_v;
            self.player.on_ground = false;
        }

        // Gravity
        self.player.vy -= gravity * dt;

        // Y-Bewegung
        let new_y = self.player.y + self.player.vy * dt;

        // Kollision nur auf Y testen
        if !self.collides_at(self.player.x, new_y, self.player.z) {
            self.player.y = new_y;
            self.player.on_ground = false;
        } else {
            // Wenn wir nach unten fallen und kollidieren -> auf Boden stehen
            if self.player.vy < 0.0 {
                self.player.on_ground = true;
            }
            // Stop vertikale Bewegung bei Kollision
            self.player.vy = 0.0;

            // Mini-Fix gegen Einsinken durch Rundung
            let mut y_fix = self.player.y;
            for _ in 0..5 {
                if !self.collides_at(self.player.x, y_fix, self.player.z) {
                    break;
                }
                y_fix += 0.01;
            }
            self.player.y = y_fix;
        }
    }

    fn collides_at(&self, px: f32, py: f32, pz: f32) -> bool {
        // Player-Hitbox (Minecraft-ish)
        let half_w = 0.3_f32; // Breite ~0.6
        let height = 1.8_f32; // Höhe ~1.8

        let min_x = px - half_w;
        let max_x = px + half_w;
        let min_y = py;
        let max_y = py + height;
        let min_z = pz - half_w;
        let max_z = pz + half_w;

        let x0 = min_x.floor() as i32;
        let x1 = max_x.floor() as i32;
        let y0 = min_y.floor() as i32;
        let y1 = max_y.floor() as i32;
        let z0 = min_z.floor() as i32;
        let z1 = max_z.floor() as i32;

        for y in y0..=y1 {
            for z in z0..=z1 {
                for x in x0..=x1 {
                    if self.world.is_solid(x, y, z) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn try_step_up(&mut self, new_x: f32, new_z: f32) -> bool {
        // Wie hoch darf "hochgesteppt" werden?
        let step_height = 0.51_f32;

        // Versuch: erst +step_height hoch, dann die Bewegung durchführen
        let y_up = self.player.y + step_height;

        // 1) Platz über uns frei?
        if self.collides_at(self.player.x, y_up, self.player.z) {
            return false;
        }

        // 2) Zielposition in der Luft frei?
        if self.collides_at(new_x, y_up, new_z) {
            return false;
        }

        // 3) Erfolg: hochsetzen + bewegen
        self.player.y = y_up;
        self.player.x = new_x;
        self.player.z = new_z;
        true
    }

    pub fn apply_input(&mut self, input: InputState) {
        // 1) Raycast, um Ziel zu bestimmen
        let (sx, sy, sz) = self.player.eye_pos();
        let (dx, dy, dz) = self.player.dir();
        let hit = self.world.raycast_first_solid(sx, sy, sz, dx, dy, dz, 20.0);
        let Some((x, y, z, block, (nx, ny, nz))) = hit else {
            if input.break_block || input.place_block {
                println!("INPUT: no target");
            }
            return;
        };

        // 2) Commands erzeugen
        if input.break_block {
            self.commands.push(Command::Break { x, y, z });
            println!("INPUT: break {:?} at ({},{},{})", block, x, y, z);
        }

        if input.place_block {
            self.commands.push(Command::Place {
                x: x + nx,
                y: y + ny,
                z: z + nz,
                block: Block::Stone,
            });
            println!("INPUT: place Stone at ({},{},{})", x + nx, y + ny, z + nz);
        }
    }

    pub fn tick(&mut self, input: InputState) {
        self.tick += 1;
        self.world.tick();
        // Movement pro Tick anwenden (halten)
        self.apply_movement(input);
        self.apply_vertical_physics(input);

        // Debug: alle 20 Ticks Raycast-Ergebnis und Position ausgeben
        if self.tick % 20 == 0 {
            println!(
                "POS x={:.2} y={:.2} z={:.2} vy={:.2} ground={}",
                self.player.x, self.player.y, self.player.z, self.player.vy, self.player.on_ground
            );
        }

        self.apply_input(input);

        // --- Commands ausführen ---
        for cmd in self.commands.drain(..) {
            match cmd {
                Command::Break { x, y, z } => {
                    let ok = self.world.break_block(x, y, z);
                    println!("CMD Break ({},{},{}) -> {}", x, y, z, ok);
                }
                Command::Place { x, y, z, block } => {
                    let ok = self.world.place_block(x, y, z, block);
                    println!("CMD Place {:?} ({},{},{}) -> {}", block, x, y, z, ok);
                }
            }
        }
    }

    pub fn world_size(&self) -> i32 {
        self.world.size()
    }

    pub fn highest_solid_in_column(&self, x: i32, z: i32) -> Option<Block> {
        let size = self.world.size();
        for y in (0..size).rev() {
            if let Some(b) = self.world.get_block_opt(x, y, z) {
                if b != Block::Air {
                    return Some(b);
                }
            }
        }
        None
    }

    pub fn player_xz(&self) -> (f32, f32) {
        (self.player.x, self.player.z)
    }

    pub fn player_dir_xz(&self) -> (f32, f32) {
        let (dx, _dy, dz) = self.player.dir();
        (dx, dz)
    }

    pub fn target_block(&self) -> Option<(i32, i32, i32)> {
        let (sx, sy, sz) = self.player.eye_pos();
        let (dx, dy, dz) = self.player.dir();
        self.world
            .raycast_first_solid(sx, sy, sz, dx, dy, dz, 20.0)
            .map(|(x, y, z, _b, _n)| (x, y, z))
    }

    pub fn unload_chunk(&mut self, pos: ChunkPos) -> bool {
        let removed = self.world.unload_chunk(pos);
        if removed {
            self.chunk_mesh_cache.remove(&pos);
        }
        removed
    }

    pub fn maintain_chunk_window(&mut self, radius: i32) {
        // Spieler-Chunk
        let player_chunk = ChunkPos {
            cx: chunk_coord(self.player.x.floor() as i32),
            cy: chunk_coord(self.player.y.floor() as i32),
            cz: chunk_coord(self.player.z.floor() as i32),
        };

        // 1) Alle Chunks im Radius (nur XZ) sicherstellen, Y-Ebene des Spielers
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let cp = ChunkPos {
                    cx: player_chunk.cx + dx,
                    cy: player_chunk.cy,
                    cz: player_chunk.cz + dz,
                };
                self.world.ensure_chunk(cp);
            }
        }

        // 2) Außerhalb entladen (nur XZ-Entfernung)
        let keep_sq = radius * radius;
        let to_unload: Vec<ChunkPos> = self
            .world
            .chunk_positions()
            .into_iter()
            .filter(|cp| {
                let dx = cp.cx - player_chunk.cx;
                let dz = cp.cz - player_chunk.cz;
                dx * dx + dz * dz > keep_sq || cp.cy != player_chunk.cy
            })
            .collect();

        for cp in to_unload {
            self.unload_chunk(cp);
        }
    }

    pub fn mesh_loaded_chunks_if_dirty(
        &mut self,
        screen_width: u32,
        screen_height: u32,
    ) -> Option<(Vec<Vertex>, Vec<u32>)> {
        let cps = self.world.chunk_positions();

        // 1) Dirty Chunks neu meshen (oder wenn noch nicht im Cache)
        let mut any_changed = false;

        for &cp in &cps {
            let was_dirty = self.world.take_chunk_dirty(cp);
            let missing = !self.chunk_mesh_cache.contains_key(&cp);

            if was_dirty || missing {
                if missing {
                    // neuer Chunk -> Nachbarn neu meshen lassen, damit Grenz-Faces verschwinden
                    const NEIGHBORS: [(i32, i32, i32); 6] = [
                        (1, 0, 0),
                        (-1, 0, 0),
                        (0, 1, 0),
                        (0, -1, 0),
                        (0, 0, 1),
                        (0, 0, -1),
                    ];
                    for (dx, dy, dz) in NEIGHBORS {
                        self.world.mark_dirty(ChunkPos {
                            cx: cp.cx + dx,
                            cy: cp.cy + dy,
                            cz: cp.cz + dz,
                        });
                    }
                }

                let (v, i) = mesh_chunk(&self.world, cp);
                self.chunk_mesh_cache.insert(cp, (v, i));
                any_changed = true;
            }
        }

        // Cache aufraeumen: Meshes zu entladenen Chunks entfernen
        self.chunk_mesh_cache
            .retain(|cp, _| self.world.has_chunk(*cp));

        if !any_changed {
            return None;
        }

        // 2) Aus Cache ein Gesamtmesh bauen (Chunk-FOV-Culling)
        let aspect = (screen_width.max(1) as f32) / (screen_height.max(1) as f32);
        let cam_pos = vec3_from(self.player.eye_pos());
        let cam_dir = vec3_from(self.player.dir()).normalize_or_zero();

        let mut verts: Vec<Vertex> = Vec::new();
        let mut inds: Vec<u32> = Vec::new();

        for cp in cps {
            if !chunk_in_frustum(cp, cam_pos, cam_dir, aspect) {
                continue;
            }
            if let Some((v, i)) = self.chunk_mesh_cache.get(&cp) {
                let base = verts.len() as u32;
                verts.extend_from_slice(v);
                inds.extend(i.iter().map(|idx| idx + base));
            }
        }

        if inds.is_empty() || verts.is_empty() {
            return Some((Vec::new(), Vec::new())); // signalisiert leeres Mesh zum Zurücksetzen
        }

        Some((verts, inds))
    }

    pub fn camera_pos_dir(&self) -> ((f32, f32, f32), (f32, f32, f32)) {
        (self.player.eye_pos(), self.player.dir())
    }
}

#[inline]
fn vec3_from(t: (f32, f32, f32)) -> Vec3 {
    Vec3::new(t.0, t.1, t.2)
}

fn chunk_bounds(cp: ChunkPos) -> (Vec3, Vec3, Vec3, f32) {
    let base = Vec3::new(
        (cp.cx * CHUNK_SIZE) as f32,
        (cp.cy * CHUNK_SIZE) as f32,
        (cp.cz * CHUNK_SIZE) as f32,
    );
    let size = Vec3::splat(CHUNK_SIZE as f32);
    let center = base + size * 0.5;
    let radius = (size * 0.5).length() * 1.02; // kleine Reserve gegen harte Schnitte
    (base, base + size, center, radius)
}

fn chunk_in_frustum(cp: ChunkPos, cam_pos: Vec3, cam_dir: Vec3, aspect: f32) -> bool {
    let (_min, _max, center, radius) = chunk_bounds(cp);

    // Distanz-Cull gegen Far-Plane (Gfx nutzt 200.0)
    let to_center = center - cam_pos;
    let dist = to_center.length();
    if dist - radius > CAMERA_FAR {
        return false;
    }

    // Wenn Kamera im Chunk oder sehr nah: immer sichtbar
    if dist < radius {
        return true;
    }

    let dir_to = to_center / dist.max(1e-6);

    // FOV-Halbwinkel
    let half_v = 0.5 * CAMERA_FOV_Y;
    let half_h = (aspect * half_v.tan()).atan(); // tan(h/2) = aspect * tan(v/2)

    // Basisachsen
    let up = Vec3::Y;
    let mut right = cam_dir.cross(up);
    if right.length_squared() < 1e-5 {
        right = Vec3::new(1.0, 0.0, 0.0); // Fallback wenn Blick senkrecht nach oben/unten
    }
    let right = right.normalize();

    let ang_allow = (radius / dist).atan(); // erlaubt etwas Spielraum fuer Chunk-Groesse

    // Horizontal (XZ)
    let cam_forward_h = (cam_dir - up * cam_dir.dot(up)).normalize_or_zero();
    let dir_h = (dir_to - up * dir_to.dot(up)).normalize_or_zero();
    if cam_forward_h.length_squared() > 0.0 && dir_h.length_squared() > 0.0 {
        let cos_h = cam_forward_h.dot(dir_h).clamp(-1.0, 1.0);
        let ang_h = cos_h.acos();
        if ang_h > half_h + ang_allow {
            return false;
        }
    }

    // Vertikal (Pitch)
    let cam_forward_v = (cam_dir - right * cam_dir.dot(right)).normalize_or_zero();
    let dir_v = (dir_to - right * dir_to.dot(right)).normalize_or_zero();
    if cam_forward_v.length_squared() > 0.0 && dir_v.length_squared() > 0.0 {
        let cos_v = cam_forward_v.dot(dir_v).clamp(-1.0, 1.0);
        let ang_v = cos_v.acos();
        if ang_v > half_v + ang_allow {
            return false;
        }
    }

    true
}
