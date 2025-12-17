//! contains various informational methods for BattleState.
use fixedstr::str32;

use crate::prelude::*;

impl BattleState {
    pub(crate) fn get_blocked_positions(&self) -> Vec<BattlePosition> {
        //! returns a vec of all positions that are either 
        //! 1) being held by stationary units 
        //! 2) being moved to by moving units
        self.live_units.iter().map(|(id, unit)|
                if let Some(movement) = unit.current_movement {
                    movement.end_pos
                } else {
                    unit.position
                }
            ).collect()
    }

    pub(crate) fn get_positions_by_team(&self, team: Team, tick: u32) -> Vec<(EntityID, BattlePosition)> {
        self.live_units.iter()
                        .filter(|unit| unit.1.team == team)
                        .map(|unit| (unit.1.id, unit.1.get_position(tick)))
                        .collect()
    }

    pub(crate) fn get_opponent_positions(&self, id: EntityID, tick: u32)  -> Vec<(EntityID, BattlePosition)> {
        let team = self.live_units.get(&id).unwrap().team;
        self.get_positions_by_team(team.opponent(), tick)
    }

    pub(crate) fn get_name(&self, id: EntityID) -> str32 {
        self.live_units.get(&id).unwrap().unit.get_name()
    }
}