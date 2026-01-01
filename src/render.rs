use crate::block::Block;
use crate::game::Game;

pub struct DebugRenderer {
    pub width: u32,
    pub height: u32,
}

impl DebugRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn draw(&self, frame: &mut [u8], game: &Game) {
        // Clear
        for px in frame.chunks_exact_mut(4) {
            px[0] = 18;
            px[1] = 18;
            px[2] = 20;
            px[3] = 255;
        }

        let size = game.world_size() as i32;

        // Grid size in pixels
        let cell = 12i32;
        let grid = size * cell;

        let off_x = (self.width as i32 - grid).max(0) / 2;
        let off_y = (self.height as i32 - grid).max(0) / 2;

        // Draw world (top-down): show highest solid block per (x,z)
        for z in 0..size {
            for x in 0..size {
                let b = game.highest_solid_in_column(x, z);
                let (r, g, bl) = match b {
                    None => (25, 25, 30),
                    Some(Block::Dirt) => (110, 80, 45),
                    Some(Block::Stone) => (130, 130, 135),
                    Some(Block::Air) => (25, 25, 30),
                };

                let px0 = off_x + x * cell;
                let py0 = off_y + z * cell;
                self.fill_rect(frame, px0, py0, cell, cell, r, g, bl);
            }
        }

        // Target highlight (raycast hit)
        if let Some((tx, _ty, tz)) = game.target_block() {
            let px0 = off_x + tx * cell;
            let py0 = off_y + tz * cell;
            self.rect_outline(frame, px0, py0, cell, cell, 255, 230, 120);
        }

        // Player
        let (px, pz) = game.player_xz();
        let pxi = off_x + (px * cell as f32) as i32;
        let pzi = off_y + (pz * cell as f32) as i32;
        self.fill_rect(frame, pxi - 2, pzi - 2, 5, 5, 80, 200, 255);

        // Direction line (simple)
        let (dx, dz) = game.player_dir_xz();
        let len = (dx * dx + dz * dz).sqrt().max(0.0001);
        let dx = dx / len;
        let dz = dz / len;

        let steps = 30;
        let mut last_x = pxi;
        let mut last_y = pzi;
        for i in 1..=steps {
            let t = i as f32 * 0.35;
            let lx = off_x + ((px + dx * t) * cell as f32) as i32;
            let ly = off_y + ((pz + dz * t) * cell as f32) as i32;
            self.line(frame, last_x, last_y, lx, ly, 255, 80, 80);
            last_x = lx;
            last_y = ly;
        }
    }

    fn put_px(&self, frame: &mut [u8], x: i32, y: i32, r: u8, g: u8, b: u8) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
        frame[idx] = r;
        frame[idx + 1] = g;
        frame[idx + 2] = b;
        frame[idx + 3] = 255;
    }

    fn fill_rect(&self, frame: &mut [u8], x: i32, y: i32, w: i32, h: i32, r: u8, g: u8, b: u8) {
        for yy in 0..h {
            for xx in 0..w {
                self.put_px(frame, x + xx, y + yy, r, g, b);
            }
        }
    }

    fn rect_outline(&self, frame: &mut [u8], x: i32, y: i32, w: i32, h: i32, r: u8, g: u8, b: u8) {
        for xx in 0..w {
            self.put_px(frame, x + xx, y, r, g, b);
            self.put_px(frame, x + xx, y + h - 1, r, g, b);
        }
        for yy in 0..h {
            self.put_px(frame, x, y + yy, r, g, b);
            self.put_px(frame, x + w - 1, y + yy, r, g, b);
        }
    }

    fn line(&self, frame: &mut [u8], x0: i32, y0: i32, x1: i32, y1: i32, r: u8, g: u8, b: u8) {
        // Bresenham
        let mut x0 = x0;
        let mut y0 = y0;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.put_px(frame, x0, y0, r, g, b);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }
}
