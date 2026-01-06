//! contains various informational methods for BattleState.

use crate::core_structs::prelude::*;

use fixedstr::str32;



impl BattleState {
    pub(crate) fn get_positions_by_team(&self, team: Team, tick: u32) -> Vec<(EntityID, BattleSubtile)> {
        self.live_units.iter()
                        .filter(|unit| unit.team == team)
                        .map(|unit| (unit.id, unit.get_position(tick)))
                        .collect()
    }

    pub(crate) fn get_opponent_positions(&self, id: EntityID, tick: u32)  -> Vec<(EntityID, BattleSubtile)> {
        let team = self.live_units.get(&id).unwrap().team;
        self.get_positions_by_team(team.opponent(), tick)
    }

    pub(crate) fn get_name(&self, id: EntityID) -> str32 {
        self.live_units.get(&id).unwrap().template.get_name()
    }
}