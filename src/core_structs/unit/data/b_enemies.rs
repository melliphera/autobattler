//! Contains the character roster itself.
use super::super::template::UnitTemplate;
use super::super::primitives::*;
use fixedstr::str32;

pub(crate) static ENEMY_DATABASE: &[UnitTemplate] = &[
    UnitTemplate {
        _id: 0,
        name: str32::const_make("Slime"),
        hitpoints: Hitpoints(150),
        defence: Mitigation(2),
        magic_resist: Mitigation(0),
        attack: AttackDamage(10),
        attack_type: Physical,
        attack_delay: AttackTickDelay(90),  // 1.5s attack interval (slow but tanky)
        attack_range: AttackRange(1),
        crit_chance: CritChance::from_percentage(10.0),
        move_speed: SecondsPerTile(0.6)     // 0.6s per tile (slightly slow)
    },
    UnitTemplate {
        _id: 1,
        name: str32::const_make("Rat"),
        hitpoints: Hitpoints(60),
        defence: Mitigation(0),
        magic_resist: Mitigation(0),
        attack: AttackDamage(6),
        attack_type: Physical,
        attack_delay: AttackTickDelay(40),  // 0.67s attack interval (fast attacker)
        attack_range: AttackRange(1),
        crit_chance: CritChance::from_percentage(10.0),
        move_speed: SecondsPerTile(0.4)     // 0.4s per tile (quick)
    },
    
];