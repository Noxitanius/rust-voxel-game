#[derive(Debug)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,

    // Blickwinkel in Radiant
    pub yaw: f32,
    pub pitch: f32,

    pub vy: f32, // vertikale Geschwindigkeit (für Springen/Fallen)
    pub on_ground: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            x: 3.5,
            y: 1.0,
            z: 3.5,
            yaw: 0.0,
            pitch: 0.35,
            vy: 0.0,
            on_ground: false,
        }
    }

    pub fn eye_pos(&self) -> (f32, f32, f32) {
        (self.x, self.y + 0.9, self.z)
    }

    pub fn dir(&self) -> (f32, f32, f32) {
        // yaw: links/rechts, pitch: hoch/runter
        let cy = self.yaw.cos();
        let sy = self.yaw.sin();
        let cp = self.pitch.cos();
        let sp = self.pitch.sin();

        // Vorwärtsrichtung
        let dx = sy * cp;
        let dy = -sp;
        let dz = cy * cp;

        (dx, dy, dz)
    }

    pub fn add_look(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch += delta_pitch;

        // clamp pitch (nicht über Kopf drehen)
        let limit = 1.55; // ~89°
        if self.pitch > limit {
            self.pitch = limit;
        }
        if self.pitch < -limit {
            self.pitch = -limit;
        }
    }
}
