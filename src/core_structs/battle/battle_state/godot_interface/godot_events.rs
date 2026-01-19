// condensed form of BattleEvent suitable for streaming to Godot.

use crate::prelude::*;

use std::fmt::Display;

#[derive(Debug)]
pub enum GodotEvent { // comment format: uses; required info
    Attack(GodotAttackData),           // DONE animation + hitsplat + health adjustment; attacker, target, post-mitigation damage.
    AbilityCast(GodotAbilityData),     // DONE just used for animations.

    DamageTaken(EntityID, Hitpoints),  // for health bar adjustment and hitsplats
    DamageHealed(EntityID, Hitpoints), // DONE as above
    Shielded(EntityID, Hitpoints),     // DONE as above, target recieves shield.

    Death(EntityID),                   // DONE

    Move(EntityID, GridPosition, u32)  // DONE for movement (duh)
    // MoveEnd not needed as that's just for logic

    // by putting ability receiver animations in, BuffEvent can be skipped entirely
}

#[derive(Debug)]
pub struct GodotAttackData {
    // for animation + hitsplats
    pub source: EntityID,
    pub target: EntityID,
    pub target_damage: Hitpoints
}

#[derive(Debug)]
pub struct GodotAbilityData {
    // purely for animation. Internal AbilityEvent spawns other events which are replicated here 
    pub source: EntityID,
    pub targets: Vec<EntityID>
}

impl Display for GodotEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Attack(data) =>              { write!(f, "{} attacked {} for {} damage.", data.source, data.target, data.target_damage.0)},
            Self::AbilityCast(data) =>         { write!(f, "{} cast their ability, targeting {:?}.", data.source, data.targets)},
            Self::DamageTaken(id, damage) =>   { write!(f, "{} took {} damage.", id, damage.0)},
            Self::DamageHealed(id, healed) =>  { write!(f, "{} was healed for {} health.", id, healed.0)},
            Self::Shielded(id, shielded ) =>   { write!(f, "{} was shielded for {} health.", id, shielded.0)},
            Self::Death(id) =>                 { write!(f, "{} died.", id)},
            Self::Move(id, endpos, endtick) => { write!(f, "{} started moving to {} and will arrive on tick {}", id, endpos, endtick)},
        }
    }
}
