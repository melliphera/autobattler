//! helper tools included to ensure game design is balanced.p
pub mod profiling;

use crate::core_structs::{prelude::*, unit::spatial_functions::TICKS_PER_SECOND};

fn _calc_dps(unit: &BattleUnit) -> f32 {
    // very incorrect for units that have self-buffs/passives for now.
    let damage_per_auto = (unit.attack.0 as f32)*(1f32+unit.crit_chance.to_percentage());

    if let Some(ability) = unit.ability {
        let auto_delays_per_cast_cycle = (unit.max_mana.0/10) + 1; // +1 bc cast also uses autodelay
        let ticks_per_cast_cycle = auto_delays_per_cast_cycle as f32 * unit.attack_delay.0 as f32;
        let attack_damage_per_cycle = (auto_delays_per_cast_cycle-1) as f32 * damage_per_auto;
        
        let mut ability_damage = 0f32;
        for effect in ability.effects.iter().filter(|a| !a.is_none()) {
            match effect.unwrap() {
                AbilityPayload::Attack(a, _b) => {
                    let num_targets = ability.target_paradigm._get_target_count();
                    ability_damage += a.0 as f32 * num_targets as f32
                }
                AbilityPayload::BuffPayload(_buff) => {}
                _ => {}
            }
        };
        (attack_damage_per_cycle + ability_damage) * TICKS_PER_SECOND as f32 / ticks_per_cast_cycle
    } else {
        damage_per_auto * 20.0 / unit.attack_delay.0 as f32
    }
}