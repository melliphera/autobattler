use crate::prelude::*;

impl BattleState {
    pub fn with_godot_event_buffer(mut self) -> Self {
        self.godot_event_buffer = Some(Vec::with_capacity(150)); // number of events per sim in random fight in profile()
        self.unit_manifest      = Some(Vec::with_capacity(20));  // contained at the top of the logfile for clarity.
        self
    }
}