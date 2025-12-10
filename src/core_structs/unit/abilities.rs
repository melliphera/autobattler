use fixedstr::str32;

use crate::core_structs::unit::prelude::*;

use super::targeting::TargetParadigm;

pub enum EffectType {
    Attack(Hitpoints, DamageType), Shield(Hitpoints), Heal(Hitpoints), BuffContainer(Buff), Move
}

use EffectType::*;

pub enum TargetTeam {
    Ally, Enemy, Oneself
}

pub struct Ability {
    pub name: str32,
    pub mana_cost: i32,
    pub effect: EffectType, // what Event this produces when it resolves
    pub target: TargetTeam, // whether it targets ally or opponent (relative to itself) - also has Oneself for short-circuiting targeting. Do not use Oneself for AoE.
    pub target_closure: TargetParadigm,  // enum wrapper for targeting function that decides who is targeted within TargetTeam - could be lowest health, highest armour etc.
    pub cast_delay: Option<u32>          // ticks between effect starting and impacts applying. Option so that instant = cast_delay: None rather than 0 
                                         // feels more explict and has behavioural distinctions (1 event rather than 2).
}

pub static ABILITY_DATABASE: &[Ability] = &[
    Ability{ 
        // takes 3 damage off all incoming attacks until the end of the fight.
        name: str32::const_make("Bulwark"),
        mana_cost: 90,
        effect: BuffContainer(Buff {
            buff_type: super::buffs_debuffs::BuffType::FlatIncomingReduction(Hitpoints(3)),
            priority: 0, // first thing
            counter: None
        }),
        target: TargetTeam::Oneself,
        target_closure: TargetParadigm::Me,
        cast_delay: None
    }
];

