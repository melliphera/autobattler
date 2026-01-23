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
            last_read: 0,
        }
    }

    pub fn push(&mut self, event: GodotEvent) {
        self.events.push(event);
    }

    pub fn diff_to_present(&mut self, unit_list: &EntityList<25>, tick: Tick) -> BattleDiff {
        let new_events = self.events[self.last_read..].to_vec();
        self.last_read = self.events.len();

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

        BattleDiff {
            events: new_events,
            in_transit: transit_positions,
        }
    }
}
