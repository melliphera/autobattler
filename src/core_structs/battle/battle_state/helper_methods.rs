//! contains various informational methods for BattleState.

use crate::core_structs::prelude::*;

use fixedstr::str32;
use smallvec::SmallVec;

pub const POSITIONS_SMALLVEC_SIZE: usize = 10;

impl BattleState {
    pub(crate) fn get_positions_by_team(&self, team: Team, tick: u32) -> SmallVec<[(EntityID, BattleSubtile); POSITIONS_SMALLVEC_SIZE]> {
        let mut out = SmallVec::<[(EntityID, BattleSubtile); POSITIONS_SMALLVEC_SIZE]>::new();

        for unit in self.live_units.iter() {
            if unit.team == team {
                out.push((unit.id, unit.get_position(tick)))
            }
        }
        out
    }

    pub(crate) fn get_opponent_positions(&self, id: EntityID, tick: u32)  -> SmallVec<[(EntityID, BattleSubtile); POSITIONS_SMALLVEC_SIZE]> {
        let team = self.live_units.get(&id).unwrap().team;
        self.get_positions_by_team(team.opponent(), tick)
    }

    pub(crate) fn get_name(&self, id: EntityID) -> str32 {
        self.live_units.get(&id).unwrap().template.get_name()
    }
}