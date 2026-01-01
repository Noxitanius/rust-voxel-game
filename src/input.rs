#[derive(Debug, Default, Clone, Copy)]
pub struct InputState {
    // --- One-shot actions (werden nach Tick zurückgesetzt) ---
    pub break_block: bool,
    pub place_block: bool,
    pub jump: bool,
    pub toggle_mouse_lock: bool,

    // --- Held keys (bleiben true solange gedrückt) ---
    pub move_fwd: bool,
    pub move_back: bool,
    pub move_left: bool,
    pub move_right: bool,
}

impl InputState {
    /// Nach jedem Tick aufrufen: setzt nur One-shot Aktionen zurück.
    pub fn clear_one_shots(&mut self) {
        self.break_block = false;
        self.place_block = false;
        self.jump = false;
        self.toggle_mouse_lock = false;
    }
}
