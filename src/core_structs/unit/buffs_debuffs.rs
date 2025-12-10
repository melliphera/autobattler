use crate::core_structs::unit::prelude::Hitpoints;
use BuffType::*;

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuffType {
    AttackDamageModifier,
    FlatIncomingReduction(Hitpoints)
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Buff {
    pub buff_type: BuffType,
    pub priority: i8,  // for sequencing.
    pub counter: Option<u32>,  // either ticks remaining or activation instances (hits blocked, attacks empowered etc) remaining, depending on the buff. If None, permanent.
}

impl Buff {
    pub fn modify(&self, damage: Hitpoints) -> Hitpoints {
        match self.buff_type {
            FlatIncomingReduction(n) => { Hitpoints(damage.0-n.0) }
            _ => damage
        }
        
    }
}