use super::*;
use crate::{core_structs::battle::battle_state::operation::EventReturnBuffer, prelude::*};


impl BattleState {
    pub fn with_godot_event_buffer(mut self) -> Self {
        self.godot_event_buffer = Some(godot_event_log::GodotEventLog::with_capacity(150)); // number of events per sim in random fight in profile()
        self.unit_manifest      = Some(Vec::with_capacity(20));  // contained at the top of the logfile for clarity.
        self
    }

    pub fn execute_live(&mut self, ticks_per_sec: u16, ticks_per_print: u16) {
        // run battle to completion, via incremental ticks.
        let mut buffer = EventReturnBuffer::new();
        let mut next_tick = 0;
        let sleep_length_ms = ticks_per_print as f32 / ticks_per_sec as f32 * 1000.0;

        self.initialize();
        let mut diff = self.godot_event_log_diff_to_present().expect("First diff failed. Is the listener on?");
        for event in diff.events.iter() {
            println!("{}", event);
        }
        for unit_pos in diff.in_transit.iter() {
            println!("Moving unit {} @ {}", unit_pos.id, unit_pos.location);
        }

        while self.timeline.len() > 0 {
            self.run_to_tick(Tick(next_tick), &mut buffer);
            next_tick += ticks_per_print;
            diff = self.godot_event_log_diff_to_present().unwrap();
            for event in diff.events.iter() {
                println!("{}", event);
            }
            for unit_pos in diff.in_transit.iter() {
                println!("Moving unit {} @ {}", unit_pos.id, unit_pos.location);
            }

            std::thread::sleep(std::time::Duration::from_millis(sleep_length_ms as u64));
        }
    }

    fn godot_event_log_diff_to_present(&mut self) -> Option<godot_event_log::BattleDiff> {
        match &mut self.godot_event_buffer {
            Some(buffer) => Some(buffer.diff_to_present(&self.live_units, self.last_processed_tick)),
            None => None,
        }
    }

    fn run_to_tick(&mut self, target_tick: Tick, buffer: &mut EventReturnBuffer) {
        // used for live running with godot event log.
        while let Some(event_container) = self.timeline.pop() {
            if event_container.tick > target_tick {
                self.timeline.push(event_container.event, event_container.tick, target_tick);
                break;
            }

            self.execute_event(event_container.event, event_container.tick, buffer);
            for (event, tick) in buffer.0.iter() {       // add newly produced items to the queue.
                self.queue_event(*event, *tick);
            }
            buffer.clear();
        }
    }
}

#[cfg(test)]
pub mod tests {
    #[test]
    fn live_execute_test() {
        use rand::Rng;
        use crate::prelude::*;
        use crate::dev_tools::profiling::TEST_SEED;
        use Roster::*;

        let mut human_team: Vec<(u16, (i32, i32))> = Vec::new();
        let mut npc_team: Vec<(u16, (i32, i32))> = Vec::new();

        let mut rng = rand::rng();

        let human_count = rng.random_range(1..=2);
        for _ in 0..human_count {
            let id = rng.random_range(0..=2);
            let x  = rng.random_range(0..=4);
            let y  = rng.random_range(0..=4);
            human_team.push((id, (x, y)));
        }

        let enemy_count = human_count + 1;
        for _ in 0..enemy_count {
            let id = rng.random_range(0..=1);
            let x  = rng.random_range(5..=9);
            let y  = rng.random_range(0..=4);
            npc_team.push((id, (x, y)));
        }

        let mut b = BattleState::new_seeded(TEST_SEED).with_godot_event_buffer();

        for (id, (x, y)) in human_team.iter() {
            _ = b.spawn_ally_from_id(UnitTemplateID(*id, Human), GridPosition { x: *x, y: *y });
        }

        for (id, (x, y)) in npc_team.iter() {
            _ = b.spawn_enemy_from_id(UnitTemplateID(*id, NPC),  GridPosition { x: *x, y: *y });
        }

        b.execute_live(20, 1);
    }
}