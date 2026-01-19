//! contains various informational methods for BattleState.
use crate::core_structs::{battle::battle_state::team_position_caches::{LocationTag, CacheCleanliness::*}, prelude::*};

use fixedstr::str32;
use smallvec::SmallVec;


pub const MAX_TEAM_SIZE: usize = 16;

impl BattleState {
    pub(crate) fn get_positions_by_team(&self, team: Team, tick: u32) -> SmallVec<[LocationTag; MAX_TEAM_SIZE]> {
        let cache = match team {
            Team::Player    => {&self.ally_pos_cache}
            Team::Opponent  => {&self.opp_pos_cache}
        };

        if let Some(pos_data) = cache.borrow().get_positions(tick) {
            return pos_data.clone();
        }
        
        let mut out = SmallVec::<[LocationTag; MAX_TEAM_SIZE]>::new();
        let mut any_moving = false; // to assert cache cleanliness

        for unit in self.live_units.iter() {
            if unit.team == team {
                let upos = unit.get_position(tick);
                any_moving = any_moving || upos.1;
                out.push(LocationTag {id: unit.id, location: upos.0})
            }
        }
        
        if any_moving {
            cache.borrow_mut().update(out.clone(), DirtyAfter(tick));
        } else {
            cache.borrow_mut().update(out.clone(), Clean);
        }
        out
    }

    pub(crate) fn get_opponent_positions(&self, id: EntityID, tick: u32)  -> SmallVec<[LocationTag; MAX_TEAM_SIZE]> {
        let team = self.live_units.get(&id).unwrap().team;
        self.get_positions_by_team(team.opponent(), tick)
    }

    pub(crate) fn get_name(&self, id: EntityID) -> str32 {
        self.live_units.get(&id).unwrap().template.get_name()
    }
}