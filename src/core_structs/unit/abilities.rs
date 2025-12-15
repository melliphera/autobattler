use fixedstr::str32;

use crate::core_structs::battle::battle_event::{AttackData, BuffData, HealData, ShieldData};
use crate::core_structs::{battle::battle_event::BattleEvent::{self, *}, unit::prelude::*};
use crate::BattleState;
use super::targeting::TargetParadigm;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) enum AbilityPayload {
    Attack(Hitpoints, DamageType), AddShield(Hitpoints), Heal(Hitpoints), BuffPayload(Buff), Move(BattlePosition)
}

use AbilityPayload::*;
use crate::core_structs::unit::buffs_debuffs::Buff;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) enum TargetTeam {
    Ally, Enemy
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct Ability {
    pub(crate) name: str32,
    pub(crate) mana_cost: u16, // for fast typing, always grabbed as a Mana object. 
    pub(crate) effect: AbilityPayload, // what Event this produces when it resolves
    pub(crate) target: TargetTeam, // whether it targets ally or opponent (relative to itself) - also has Oneself for short-circuiting targeting. Do not use Oneself for AoE.
    pub(crate) target_paradigm: TargetParadigm,  // enum wrapper for targeting function that decides who is targeted within TargetTeam - could be lowest health, highest armour etc.
    pub(crate) cast_delay: Option<u32>          // ticks between effect starting and impacts applying. Option so that instant = cast_delay: None rather than 0 
                                         // feels more explict and has behavioural distinctions (1 event rather than 2).
}

impl Ability {
    pub(crate) fn get_mana(&self) -> Mana {
        Mana(self.mana_cost)
    }

    pub(crate) fn get_targets(&self, caster: EntityID, b: &BattleState, tick: u32) -> Vec<EntityID> {
        let ally_positions  = b.get_positions(Team::Player, tick);
        let enemy_positions = b.get_positions(Team::Opponent, tick);

        let caster_unit = b.live_units.get(&caster).expect("Caster wasn't found while casting ability.");
        let team_digit: i8 = if caster_unit.team == Team::Player {1} else {-1};
        let side_digit: i8 = if self.target == TargetTeam::Ally  {1} else {-1};
        let mut viable_targets = match team_digit*side_digit {
             1 => ally_positions,
            -1 => enemy_positions,
             _ => unreachable!()
        };

        match self.target_paradigm {
            TargetParadigm::Me => { return vec![caster]}
            
            TargetParadigm::CurrentTarget => {
                if let Some(t) = caster_unit.target {
                    vec![t]
                } else {
                    unreachable!("Caught casting Target ability with no target") // target sanitisation happens in the core BattleState logic.
                }
            }

            TargetParadigm::Nearest(n) => {
                // 1. find out which team this a
                viable_targets.sort_by_key(|target| caster_unit.get_position(tick).distance_squared_to(&target.1));
                viable_targets[0..n as usize].iter().map(|x| x.0).collect()
            }

            TargetParadigm::Furthest(n) => {
                viable_targets.sort_by_key(|target| -caster_unit.get_position(tick).distance_squared_to(&target.1).0);
                viable_targets[0..n as usize].iter().map(|x| x.0).collect()
            }
        }
    }

    pub(crate) fn create_event(&self, source: EntityID, target: EntityID) -> (BattleEvent, u32) {
        let event = match self.effect {
            Attack(damage, damage_type) => {
                RawDamageEvent(AttackData{
                    source, target, damage, damage_type
                })
            }
            AddShield(amount) => {
                ShieldEvent(ShieldData{
                    source, target, amount
                })
            }
            Heal(amount) => {
                HealEvent(HealData{
                    source, target, amount
                })
            }
            BuffPayload(buff) => {
                BuffEvent(BuffData {
                    source, target, buff
                })
            }
            Move(_pos) => {
                unimplemented!()
            }
        };
        (event, self.cast_delay.unwrap_or(0))
    }
}


