// condensed form of BattleEvent suitable for streaming to Godot.
use crate::prelude::*;

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
    pub ability_name: fixedstr::str32,
    pub targets: [EntityID; 6] //
}

#[test] 
fn test_godot_event_size() {
    use std::mem::size_of;
    let p = size_of::<GodotEvent>();
    println!("Size of GodotEvent: {}", p);
    assert!(p <= 64);
}

impl std::fmt::Display for GodotEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GodotEvent::Attack(adata)               => write!(f, "{} attacked {} for {} damage.", adata.source, adata.target, adata.target_damage.0),             
            GodotEvent::AbilityCast(adata)          => write!(f, "{} cast {}, affecting {:?}.", adata.source, adata.ability_name, adata.targets),
            GodotEvent::DamageTaken(id, amount)     => write!(f, "{} took {} damage.", id, amount.0),
            GodotEvent::DamageHealed(id, amount)    => write!(f, "{} healed for {} health.", id, amount.0),
            GodotEvent::Shielded(id, amount)        => write!(f, "{} was shielded for {} health.", id, amount.0),
            GodotEvent::Move(id, end_loc, end_tick) => write!(f, "{} started moving to {}, arriving on tick {}.", id, end_loc, end_tick),        
            GodotEvent::Death(id)                   => write!(f, "{} died.", id)
        }
    }
}