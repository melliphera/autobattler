use crate::prelude::*;

impl BattleState {
    pub fn with_godot_event_buffer(&mut self) {
        self.godot_event_buffer = Some(Vec::with_capacity(150)) // number of events per sim in random fight in profile()
    }
}