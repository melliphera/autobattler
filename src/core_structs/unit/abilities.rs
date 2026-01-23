use std::{cmp::Reverse, u16};

use fixedstr::str32;

use crate::core_structs::{battle::battle_state::{godot_interface::godot_events::{GodotAbilityData, GodotEvent}, operation::EventReturnBuffer}, prelude::*};
use super::targeting::TargetParadigm;
use AbilityPayload::*;
use crate::core_structs::unit::buffs_debuffs::Buff;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum AbilityPayload {
    Attack(Hitpoints, DamageType), _AddShield(Hitpoints), _Heal(Hitpoints, bool), BuffPayload(Buff), _Move(BattleSubtile, MoveSpeed)
}



#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum TargetTeam {
    Ally, Enemy
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Ability {
    pub(crate) name: str32,
    pub(crate) mana_cost: u16, // for fast typing, always grabbed as a Mana object. 
    pub(crate) effects: [Option<AbilityPayload>; 4], // what Event this produces when it resolves
    pub(crate) target: TargetTeam,                  // whether it targets ally or opponent (relative to itself) - also has Oneself for short-circuiting targeting. Do not use Oneself for AoE.
    pub(crate) target_paradigm: TargetParadigm,     // enum wrapper for targeting function that decides who is targeted within TargetTeam - could be lowest health, highest armour etc.
    pub(crate) cast_delay: Option<Tick>              // ticks between effect starting and impacts applying. Option so that instant = cast_delay: None rather than 0 
                                                    // (...cont) feels more explict and has behavioural distinctions (1 event rather than 2).
}

impl Ability {
    pub(crate) fn get_mana(&self) -> Mana {
        Mana(self.mana_cost)
    }

    pub(crate) fn cast(&self, caster: EntityID, b: &mut BattleState, tick: Tick, buffer: &mut EventReturnBuffer) {
        //! get_targets mutates buffer.1, and cast reads it. This buffer is usually used to manage character death, 
        //! therefore it is cleared at the end of the function.
        self.get_targets(caster, b, tick, buffer);

        let t = buffer.1.clone();

        for target in t.iter() {
            self.create_events(caster, *target, tick, buffer)
        }

        if let Some(ref mut godot_buffer) = b.godot_event_buffer {
            godot_buffer.push(GodotEvent::AbilityCast(GodotAbilityData {
                source: caster,
                ability_name: self.name,
                targets: t
            }))
        } 
        
        buffer.1.clear();
    }

    fn get_targets(&self, caster: EntityID, b: &BattleState, tick: Tick, buffer: &mut EventReturnBuffer) {
        let caster_unit = b.live_units.get(&caster).expect("Caster wasn't found while casting ability.");
        let team_digit: i8 = if caster_unit.team == Team::Player {1} else {-1};
        let side_digit: i8 = if self.target == TargetTeam::Ally  {1} else {-1};
        let mut viable_targets = match team_digit*side_digit {
             1 => b.get_positions_by_team(Team::Player, tick),
            -1 => b.get_positions_by_team(Team::Opponent, tick),
             _ => unreachable!()
        };

        match self.target_paradigm {
            TargetParadigm::Me => { buffer.1.push(caster); }
            
            TargetParadigm::CurrentTarget => {
                if let Some(t) = caster_unit.target {
                    buffer.1.push(t) } else {
                    unreachable!("Caught casting Target ability with no target") // target sanitisation happens in the core BattleState logic.
                }
            }

            TargetParadigm::Nearest(n) => {
                viable_targets.sort_by_key(|target| caster_unit.get_position(tick).0.distance_squared_to(&target.location));
                let num_targets = (n as usize).min(viable_targets.len());
                buffer.1.clear();
                for target in &viable_targets[0..num_targets] {
                    buffer.1.push(target.id);
                }
            }

            TargetParadigm::Furthest(n) => {
                viable_targets.sort_by_key(|target| Reverse(caster_unit.get_position(tick).0.distance_squared_to(&target.location)));
                let num_targets = (n as usize).min(viable_targets.len());
                buffer.1.clear();
                for target in &viable_targets[0..num_targets] {
                    buffer.1.push(target.id);
                }
            }
        }
    }

    fn create_events(&self, source: EntityID, target: EntityID, cast_tick: Tick, buffer: &mut EventReturnBuffer) {
        //! creates the BattleEvent object describing the ability's effect on a given target.
        for effect_slot in self.effects.iter() {
            if let Some(event) = effect_slot {
                let e = match event {
                    Attack(damage, damage_type) => {
                        RawDamageEvent(AttackData{
                            source, 
                            target, 
                            damage: *damage, 
                            damage_type: *damage_type, 
                            caused_by: self.name
                        })
                    }
                    _AddShield(amount) => {
                        ShieldEvent(ShieldData{
                            source, 
                            target, 
                            amount: *amount
                        })
                    }
                    _Heal(amount, can_overheal) => {
                        HealEvent(HealData{
                            source, 
                            target, 
                            amount: *amount, 
                            can_overheal: *can_overheal
                        })
                    }
                    BuffPayload(buff) => {
                        BuffEvent(BuffData {
                            source, 
                            target, 
                            buff: *buff
                        })
                    }
                    _Move(_pos, _speed) => {
                        unimplemented!()
                    }
                };
                buffer.0.push((e, cast_tick + self.cast_delay.unwrap_or(Tick(0))));
            }
        }
    }
}


