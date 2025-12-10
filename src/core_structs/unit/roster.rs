//! Contains the character roster itself.
use super::template::UnitTemplate;
use super::primitives::*;
use fixedstr::str32;

pub static UNIT_DATABASE: &[UnitTemplate] = &[
    UnitTemplate {
        id: 0,
        name: str32::const_make("Knight"),
        hitpoints: Hitpoints(250),
        defence: Mitigation(2),
        magic_resist: Mitigation(2),
        attack: AttackDamage(10),
        attack_type: Physical,
        attack_delay: AttackTickDelay(20),
        attack_range: AttackRange(1),
        crit_chance: CritChance::from_percentage(10.0),
    },
    UnitTemplate {
        id: 1,
        name: str32::const_make("Ranger"),
        hitpoints: Hitpoints(75),
        defence: Mitigation(0),
        magic_resist: Mitigation(0),
        attack: AttackDamage(10),
        attack_type:  Physical,
        attack_delay: AttackTickDelay(15),
        attack_range: AttackRange(6),
        crit_chance: CritChance::from_percentage(10.0),
    },
];