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
        attack_delay: AttackTickDelay(60),  // 1.0s attack interval
        attack_range: AttackRange(1),
        move_speed: SecondsPerTile(0.5),    // 0.5s per tile
        crit_chance: CritChance::from_percentage(10.0),
    },
    UnitTemplate {
        _id: 1,
        name: str32::const_make("Ranger"),
        hitpoints: Hitpoints(75),
        defence: Mitigation(0),
        magic_resist: Mitigation(0),
        attack: AttackDamage(10),
        attack_type: Physical,
        attack_delay: AttackTickDelay(45),  // 0.75s attack interval
        attack_range: AttackRange(6),
        move_speed: SecondsPerTile(0.5),    // 0.5s per tile
        crit_chance: CritChance::from_percentage(10.0),
    },
    UnitTemplate {
        _id: 2,
        name: str32::const_make("Mage"),
        hitpoints: Hitpoints(50),
        defence: Mitigation(0),
        magic_resist: Mitigation(0),
        attack: AttackDamage(4),
        attack_type: Magic,
        attack_delay: AttackTickDelay(50),  // 0.83s attack interval
        attack_range: AttackRange(6),
        move_speed: SecondsPerTile(0.6),    // 0.6s per tile (slightly slower)
        crit_chance: CritChance::from_percentage(10.0),
    },
    
];