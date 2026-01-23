use crate::{core_structs::battle::battle_state::{entity_list::EntityList, godot_interface::godot_events::GodotEvent, team_position_caches::LocationTag}, prelude::{BattleState, EntityID}};
use crate::prelude::*;

#[derive(Debug)]
pub struct GodotEventLog {
    pub events: Vec<GodotEvent>,
    pub in_transit: Vec<EntityID>,
    pub last_read: usize,
    
}

pub struct BattleDiff {
    pub events: Vec<GodotEvent>,
    pub in_transit: Vec<LocationTag>,
}

impl GodotEventLog {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity), // total number of events in the battle. ~150 in profiling sims.
            in_transit: Vec::with_capacity(8),    // units moving at any given moment. usually less than 8.
            last_read: 0,                         // for use in the pre-calculated communication system.
        }
    }

    pub fn push(&mut self, event: GodotEvent) {
        self.events.push(event);
    }

    pub fn diff_to_present(&mut self, unit_list: &EntityList<25>, tick: Tick) -> BattleDiff {
        // for use with the "live update" communication model, in tandem with BattleState.run_to_tick();
        let new_events = self.events.to_vec();

        let mut transit_positions = Vec::with_capacity(self.in_transit.len());
        let mut has_dead = false;

        for id in &self.in_transit {
            if let Some(living_unit) = unit_list.get(id) {
                transit_positions.push(LocationTag {
                    id: *id,
                    location: living_unit.get_position(tick).0,
                });
            } else {
                has_dead = true;
            }
        }

        // Only do retain if we found at least one dead unit.
        if has_dead {
            self.in_transit.retain(|id| unit_list.get(id).is_some());
        }
        
        // the advantage of the live update model is reduced memory footprint as events don't need to be stored.
        // therefore, dump stored events after calling.
        self.events.clear();

        BattleDiff {
            events: new_events,
            in_transit: transit_positions,
        }
    }

    pub fn diff_to_tick(&mut self, tick: Tick) {
        // for use with pre-computed GodotEventLogs using the "pre-calculate" communication model.; called by Godot directly.
        todo!();
    }
}
