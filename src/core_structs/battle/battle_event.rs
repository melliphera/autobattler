use crate::core_structs::unit::prelude::*;
use BattleEvent::*;

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BattleEvent {
    // constitutes everything a character can do in a battle.
    AttackEvent(AttackData), HealEvent(HealData), ShieldEvent(ShieldData), BuffEvent(BuffData), MoveEvent(MoveData), DeathEvent(EntityID), // TODO BufExpireEvent(BuffID)
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub struct AttackData {
    pub source: EntityID,
    pub target: EntityID,
    pub damage: Hitpoints, // calculated after attacker buffs, before defensive response
    pub damage_type: DamageType
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub struct HealData {
    pub source: EntityID,
    pub target: EntityID,
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ShieldData {
    pub source: EntityID,    
    pub target: EntityID,
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub struct BuffData {
    pub source: EntityID,
    pub target: EntityID,
    pub buff: Buff
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub struct MoveData {
    pub source: EntityID,
    pub target: EntityID,
}

impl BattleEvent {
    pub fn get_source_id(self) -> EntityID {
        match self {
            AttackEvent(data) => {data.source}
            HealEvent(data)   => {data.source}
            ShieldEvent(data) => {data.source}
            BuffEvent(data)   => {data.source}
            MoveEvent(data)   => {data.source}
            DeathEvent(data)  => {data}
        }
    }

    pub fn get_target_id(self) -> EntityID {
        match self {
            AttackEvent(data) => {data.target}
            HealEvent(data)   => {data.target}
            ShieldEvent(data) => {data.target}
            BuffEvent(data)   => {data.target}
            MoveEvent(data)   => {data.target}
            DeathEvent(data)  => {data} // bogus return but never relevant so not worth changing to Option<EntityID>
        }
    }
}

