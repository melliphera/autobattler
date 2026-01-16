// condensed form of BattleEvent suitable for streaming to Godot.

use crate::prelude::*;

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

pub struct GodotAttackData {
    // for animation + hitsplats
    pub source: EntityID,
    pub target: EntityID,
    pub target_damage: Hitpoints
}

pub struct GodotAbilityData {
    // purely for animation. Internal AbilityEvent spawns other events which are replicated here 
    pub source: EntityID,
    pub targets: Vec<EntityID>
}

#[test] 
fn test_godot_event_size() {
    use std::mem::size_of;
    let p = size_of::<GodotEvent>();
    println!("Size of GodotEvent: {}", p);
    assert!(p <= 64);
}