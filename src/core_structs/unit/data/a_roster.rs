//! Contains the character roster itself.
use super::super::template::UnitTemplate;
use super::super::primitives::*;
use fixedstr::str32;

pub(crate) static UNIT_DATABASE: &[UnitTemplate] = &[
    UnitTemplate {
        _id: 0,
        name: str32::const_make("Knight"),
        hitpoints: Hitpoints(250),
        defence: Mitigation(2),
        magic_resist: Mitigation(2),
        attack: AttackDamage(10),
        attack_type: Physical,
        attack_delay: AttackTickDelay(200),
        attack_range: AttackRange(1),
        move_speed:  MoveSpeed(128), // subtiles per tick
        crit_chance: CritChance::from_percentage(10.0),
    },
    UnitTemplate {
        _id: 1,
        name: str32::const_make("Ranger"),
        hitpoints: Hitpoints(75),
        defence: Mitigation(0),
        magic_resist: Mitigation(0),
        attack: AttackDamage(10),
        attack_type:  Physical,
        attack_delay: AttackTickDelay(150),
        attack_range: AttackRange(6),
        move_speed: MoveSpeed(128),
        crit_chance: CritChance::from_percentage(10.0),
    },
    UnitTemplate {
        _id: 2,
        name: str32::const_make("Mage"),
        hitpoints: Hitpoints(50),
        defence: Mitigation(0),
        magic_resist: Mitigation(0),
        attack: AttackDamage(4),
        attack_type:  Magic ,
        attack_delay: AttackTickDelay(150),
        attack_range: AttackRange(6),
        move_speed: MoveSpeed(64),
        crit_chance: CritChance::from_percentage(10.0),
    },
    
];