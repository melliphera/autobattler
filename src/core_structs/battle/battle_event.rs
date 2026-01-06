use crate::core_structs::prelude::*;
use BattleEvent::*;

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum BattleEvent {
    // constitutes everything that can modify state in a battle.
    // AttackEvent and AbilityEvent are caused directly by entities.
    // AbilityEvent propagates other events as appropriate.
    AttackEvent(AttackData), AbilityCastEvent(AbilityData), 
    RawDamageEvent(AttackData), HealEvent(HealData), ShieldEvent(ShieldData), BuffEvent(BuffData), MoveEvent(MoveData), MoveEndEvent(MoveEndData), DeathEvent(EntityID), // TODO BufExpireEvent(BuffID)
    _DebugEvent(DebugData)
}

pub mod data_types {
    use fixedstr::str32;

    use crate::core_structs::prelude::*;

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct AttackData {
        pub(crate) source: EntityID,
        pub(crate) target: EntityID,
        pub(crate) damage: Hitpoints, // calculated after attacker buffs, before defensive response
        pub(crate) damage_type: DamageType,
        pub(crate) caused_by: str32
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct HealData {
        pub(crate) source: EntityID,
        pub(crate) target: EntityID,
        pub(crate) amount: Hitpoints,
        pub(crate) can_overheal: bool
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct ShieldData {
        pub(crate) source: EntityID,    
        pub(crate) target: EntityID,
        pub(crate) amount: Hitpoints,
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct BuffData {
        pub(crate) source: EntityID,
        pub(crate) target: EntityID,
        pub(crate) buff: Buff
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct MoveData {
        pub(crate) source: EntityID,
        pub(crate) target: EntityID, // in case of displacing abilities
        pub(crate) start_pos: GridPosition,
        pub(crate) end_pos: GridPosition,
        pub(crate) start_tick: u32,
        pub(crate) end_tick: u32,
        pub(crate) move_speed_override: Option<MoveSpeed> // for forced displacements
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct MoveEndData {
        pub(crate) target: EntityID, // in case of displacing abilities
        pub(crate) end_pos: GridPosition,
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct AbilityData{
        pub(crate) source: EntityID,
        pub(crate) ability: Ability
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub(crate) struct DebugData {
        pub(crate) delay: u32,
        pub(crate) info: DebugInfo // what to //print for the debugevent
    }

    #[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
    pub enum DebugInfo {
        _Positions, _Health, _HealthAndPos, _EventQueue
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
            _ => None
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
            _DebugEvent(_data)      => {None}
        }
    }
}

