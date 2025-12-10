use crate::core_structs::{battle::battle_state::BattleState, unit::prelude::{BattleUnit, EntityID}};
use TargetParadigm::*;

#[derive(Copy, Clone)]
pub enum TargetParadigm {
    // common targeting paradigms.
    Me,            // caster
    CurrentTarget, // self explanatory.
    Nearest(u8),   // returns the n nearest targets
    Furthest(u8)   // returns the n furthest targets
}



impl TargetParadigm {
    fn select_targets(&self, b: &BattleState, caster: EntityID, candidates: Vec<EntityID>) -> Vec<EntityID> {
        let caster_unit = b.live_units.get(&caster).unwrap();
        match self {
            Me => {vec![caster]}
            CurrentTarget => {
                vec![caster_unit.target.expect("Failed to find caster unit while ability targeting.")]
            }
            Nearest(n) => {
                let mut candidate_units: Vec<&BattleUnit> = candidates.iter().map(|eid| b.live_units.get(eid).expect("Invalid candidate found during targeting.")).collect();
                candidate_units.sort_by_key(|cand| caster_unit.position.distance_squared_to(&cand.position));
                candidate_units[0..*n as usize].iter().map(|cand| cand.id).collect()
            }
            _ => Vec::new()
        }
    }
}