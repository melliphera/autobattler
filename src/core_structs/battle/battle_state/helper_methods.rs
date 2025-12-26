//! contains various informational methods for BattleState.
use std::cell::RefMut;

use crate::core_structs::prelude::*;

use fixedstr::str32;



impl BattleState {
    pub(crate) fn get_blocked_positions(&self) -> RefMut<'_, Vec<BattlePosition>> {
        //! returns a vec of all positions that are either 
        //! 1) being held by stationary units 
        //! 2) being moved to by moving units
        //! 3) the end tile of a queued but unprocessed event.
        if self.blocked_dirty.get() {
            
            #[cfg(test)]
            println!("Blocked cache miss.");

            let capacity = self.live_units.len() + self.timeline.len();
            {
                let mut out = self.blocked_cache.borrow_mut();
                out.clear();
                out.reserve(capacity);

                out.extend(self.timeline.iter().filter_map(|event_container| {
                    match event_container.event {
                        MoveEvent(info) => Some(info.end_pos),
                        _ => None
                    }
                }));

                out.extend(self.live_units.iter().map(|unit|
                    if let Some(movement) = unit.current_movement {
                        movement.end_pos
                    } else {
                        unit.position
                    }
                ));
            } // drop borrow_mut before returning borrow()
            self.blocked_dirty.set(false);
        } else {
            #[cfg(test)]
            println!("Blocked cache hit.")
        }
        self.blocked_cache.borrow_mut()

    }

    pub(crate) fn get_positions_by_team(&self, team: Team, tick: u32) -> Vec<(EntityID, BattlePosition)> {
        self.live_units.iter()
                        .filter(|unit| unit.team == team)
                        .map(|unit| (unit.id, unit.get_position(tick)))
                        .collect()
    }

    pub(crate) fn get_opponent_positions(&self, id: EntityID, tick: u32)  -> Vec<(EntityID, BattlePosition)> {
        let team = self.live_units.get(&id).unwrap().team;
        self.get_positions_by_team(team.opponent(), tick)
    }

    pub(crate) fn get_name(&self, id: EntityID) -> str32 {
        self.live_units.get(&id).unwrap().template.get_name()
    }
}