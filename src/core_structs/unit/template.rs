//! represents the character as they are in the roster, including base stats.

use fixedstr::str32;
use super::primitives::*;

#[derive(Copy, Clone)]
pub(crate) struct UnitTemplate {
    // represents a character in the roster. 
    pub(crate) id: usize,                     // unique identifier. As visuals are also unique, this determines which sprite to load.
    pub(crate) name: str32,                   // display name.
    pub(crate) hitpoints: Hitpoints,          // damage that can be taken before death
    pub(crate) defence: Mitigation,           // 
    pub(crate) magic_resist: Mitigation,      //
    pub(crate) attack_type: DamageType,       // whether their primary attack is physical or magic
    pub(crate) attack: AttackDamage,          // damage dealt with basic attacks
    pub(crate) attack_delay: AttackTickDelay, // number of ticks between consecutive attacks.
    pub(crate) attack_range: AttackRange,     // name descriptive. u8 wrapper
    pub(crate) crit_chance: CritChance,       // name descriptive. u16 wrapper.
    pub(crate) move_speed: MoveSpeed
}