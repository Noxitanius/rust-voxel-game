// src/camera.rs
use crate::player::Player;

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
    pub fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
    pub fn len(self) -> f32 {
        (self.dot(self)).sqrt()
    }
    pub fn norm(self) -> Vec3 {
        let l = self.len();
        if l > 1e-6 {
            Vec3::new(self.x / l, self.y / l, self.z / l)
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    // column-major 4x4 like WGSL expects
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    pub fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn mul(self, b: Mat4) -> Mat4 {
        // column-major multiply: self * b
        let mut r = [[0.0; 4]; 4];
        for c in 0..4 {
            for rrow in 0..4 {
                r[c][rrow] =
                    self.m[0][rrow] * b.m[c][0] +
                    self.m[1][rrow] * b.m[c][1] +
                    self.m[2][rrow] * b.m[c][2] +
                    self.m[3][rrow] * b.m[c][3];
            }
        }
        Mat4 { m: r }
    }

    pub fn perspective(fov_y_rad: f32, aspect: f32, z_near: f32, z_far: f32) -> Mat4 {
        let f = 1.0 / (fov_y_rad * 0.5).tan();
        let nf = 1.0 / (z_near - z_far);

        // Right-handed, clip-space Z 0..1 (wgpu)
        Mat4 {
            m: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, z_far * nf, -1.0],
                [0.0, 0.0, (z_far * z_near) * nf, 0.0],
            ],
        }
    }

    pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
        let f = center.sub(eye).norm();
        let s = f.cross(up).norm();
        let u = s.cross(f);

        // column-major
        Mat4 {
            m: [
                [s.x, u.x, -f.x, 0.0],
                [s.y, u.y, -f.y, 0.0],
                [s.z, u.z, -f.z, 0.0],
                [-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0],
            ],
        }
    }
}

pub struct Camera {
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            fov_y: 70.0_f32.to_radians(),
            z_near: 0.05,
            z_far: 200.0,
        }
    }

    pub fn view_proj(&self, player: &Player, width: u32, height: u32) -> Mat4 {
        let aspect = (width.max(1) as f32) / (height.max(1) as f32);

        let (ex, ey, ez) = player.eye_pos();
        let (dx, dy, dz) = player.dir();

        let eye = Vec3::new(ex, ey, ez);
        let center = Vec3::new(ex + dx, ey + dy, ez + dz);
        let up = Vec3::new(0.0, 1.0, 0.0);

        let view = Mat4::look_at(eye, center, up);
        let proj = Mat4::perspective(self.fov_y, aspect, self.z_near, self.z_far);

        proj.mul(view)
    }
}
