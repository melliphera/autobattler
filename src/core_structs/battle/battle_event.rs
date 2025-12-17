use crate::prelude::*;
use BattleEvent::*;

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum BattleEvent {
    // constitutes everything that can modify state in a battle.
    // AttackEvent and AbilityEvent are caused directly by entities.
    // AbilityEvent propagates other events as appropriate.
    AttackEvent(AttackData), AbilityCastEvent(AbilityData), 
    RawDamageEvent(AttackData), HealEvent(HealData), ShieldEvent(ShieldData), BuffEvent(BuffData), MoveEvent(MoveData), MoveEndEvent(MoveEndData), DeathEvent(EntityID), // TODO BufExpireEvent(BuffID)
}

pub(crate) mod data_types {
    use crate::prelude::*;

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub(crate) struct AttackData {
        pub(crate) source: EntityID,
        pub(crate) target: EntityID,
        pub(crate) damage: Hitpoints, // calculated after attacker buffs, before defensive response
        pub(crate) damage_type: DamageType
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub(crate) struct HealData {
        pub(crate) source: EntityID,
        pub(crate) target: EntityID,
        pub(crate) amount: Hitpoints
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub(crate) struct ShieldData {
        pub(crate) source: EntityID,    
        pub(crate) target: EntityID,
        pub(crate) amount: Hitpoints,
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub(crate) struct BuffData {
        pub(crate) source: EntityID,
        pub(crate) target: EntityID,
        pub(crate) buff: Buff
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub(crate) struct MoveData {
        pub(crate) source: EntityID,
        pub(crate) target: EntityID, // in case of displacing abilities
        pub(crate) start_pos: BattlePosition,
        pub(crate) end_pos: BattlePosition,
        pub(crate) start_tick: u32,
        pub(crate) end_tick: u32
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub(crate) struct MoveEndData {
        pub(crate) target: EntityID, // in case of displacing abilities
        pub(crate) end_pos: BattlePosition,
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub(crate) struct AbilityData{
        pub(crate) source: EntityID,
        pub(crate) ability: Ability
    }
}


impl BattleEvent {
    pub(crate) fn get_source_id(self) -> Option<EntityID> {
        // only used for clearing events relating to dead people.
        // Knock-on effects of successful casts shouldnt be cleared.
        match self {
            AttackEvent(data)       => {Some(data.source)}
            AbilityCastEvent(data)  => {Some(data.source)}
            MoveEndEvent(data)     =>  {Some(data.target)}
            RawDamageEvent(_data)   => {None}
            HealEvent(_data)        => {None}
            ShieldEvent(_data)      => {None}
            BuffEvent(_data)        => {None}
            MoveEvent(_data)        => {None}
            DeathEvent(_data)       => {None}
        }
    }

    pub(crate) fn get_target_id(self) -> Option<EntityID> {
        match self {
            // only used for clearing events relating to dead people.
            AttackEvent(data)       => {Some(data.target)}            
            HealEvent(data)         => {Some(data.target)}
            ShieldEvent(data)       => {Some(data.target)}
            BuffEvent(data)         => {Some(data.target)}
            MoveEvent(data)         => {Some(data.target)}
            MoveEndEvent(data)      => {Some(data.target)}
            RawDamageEvent(data)    => {Some(data.target)}
            AbilityCastEvent(_data) => {None} // abilities with targeted effects manifest them as one of the above events so dont need to be handled directly.
            DeathEvent(_data)       => {None} // bogus return but never relevant so not worth changing to Option<EntityID>
        }
    }
}

