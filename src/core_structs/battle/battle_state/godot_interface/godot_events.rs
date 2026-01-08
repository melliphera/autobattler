// condensed form of BattleEvent suitable for streaming to Godot.

use crate::prelude::*;

pub enum GodotEvent { // comment format: uses; required info
    Attack(GodotAttackData),          // animation + hitsplat + health adjustment; attacker, target, post-mitigation damage.
    AbilityCast(GodotAbilityData),    // just used for animations.

    DamageTaken(EntityID, Hitpoints),  // for health bar adjustment and hitsplats
    DamageHealed(EntityID, Hitpoints), // as above
    Shielded(EntityID, Hitpoints),     // as above, target recieves shield.


    Move(EntityID, GridPosition, MoveSpeed) // for movement (duh)
    // MoveEnd not needed as that's just for logic

    // by putting ability receiver animations in, BuffEvent can be skipped entirely
}

pub struct GodotAttackData {
    // for animation + hitsplats
    pub source: EntityID,
    pub target: EntityID,
    pub target_damage: Hitpoints
}

pub struct GodotAbilityData {
    // purely for animation. Internal AbilityEvent spawns other events which are replicated here for 
    pub source: EntityID,
    pub targets: Vec<EntityID>
}
