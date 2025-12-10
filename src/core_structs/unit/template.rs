//! represents the character as they are in the roster, including base stats.

use fixedstr::str32;
use super::primitives::*;

#[derive(Copy, Clone)]
pub struct UnitTemplate {
    // represents a character in the roster. 
    pub id: usize,                     // unique identifier. As visuals are also unique, this determines which sprite to load.
    pub name: str32,                   // display name.
    pub hitpoints: Hitpoints,          // damage that can be taken before death
    pub defence: Mitigation,           // 
    pub magic_resist: Mitigation,      //
    pub attack_type: DamageType,       // whether their primary attack is physical or magic
    pub attack: AttackDamage,          // damage dealt with basic attacks
    pub attack_delay: AttackTickDelay, // number of ticks between consecutive attacks.
    pub attack_range: AttackRange,     // name descriptive. u8 wrapper
    pub crit_chance: CritChance,       // name descriptive. u16 wrapper.
}