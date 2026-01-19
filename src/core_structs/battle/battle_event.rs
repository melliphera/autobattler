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
    pub(crate) fn is_key_event(&self) -> bool {
        //! key events are defined as those in a unit's event stream. 
        //! while the unit lives and battle continues, each key event processed spawns exactly one new key event for that unit.
        //! non-key events are side-effects of key events and do not spawn new key events.
        //! The exception to this is MoveEvent. This is a non-key event but exists as part of the key event chain.
        //! i.e AttackEvent or AbilityCastEvent can spawn a MoveEvent, which spawns a MoveEndEvent, restoring the key event chain.
        //! MoveEvent is considered non-key because it is spawned and executed on-tick - at the end of a tick a unit will never have a MoveEvent queued for it.
        match self {
            AttackEvent(_)       => true,
            AbilityCastEvent(_)  => true,
            MoveEndEvent(_)      => true,
            _ => false
        }
    }

    pub(crate) fn _get_source_id(self) -> Option<EntityID> {
        // used for logging only
        match self {
            AttackEvent(data)       => {Some(data.source)}
            AbilityCastEvent(data)  => {Some(data.source)}
            HealEvent(data)         => {Some(data.source)}
            ShieldEvent(data)       => {Some(data.source)}
            BuffEvent(data)         => {Some(data.source)}
            RawDamageEvent(data)    => {Some(data.source)}

            // remember in these cases, the "target unit" is the one moving and NOT the unit's target. 
            MoveEvent(data)         => {Some(data.target)}
            MoveEndEvent(data)      => {Some(data.target)}
            _ => None
        }
    }

    pub(crate) fn get_source_for_pruning(self) -> Option<EntityID> {
        // only used for clearing events relating to dead people.
        // Knock-on effects of successful casts shouldnt be cleared.
        match self {
            AttackEvent(data)       => {Some(data.source)}
            AbilityCastEvent(data)  => {Some(data.source)}
            MoveEndEvent(data)     =>  {Some(data.target)}
            _ => None
        }
    }

    pub(crate) fn get_target_for_pruning(self) -> Option<EntityID> {
        match self {
            // only used for clearing events relating to dead people.
            HealEvent(data)         => {Some(data.target)}
            ShieldEvent(data)       => {Some(data.target)}
            BuffEvent(data)         => {Some(data.target)}
            RawDamageEvent(data)    => {Some(data.target)}

            // remember in these cases, the target is the one moving and NOT the unit's target.
            MoveEvent(data)         => {Some(data.target)} 
            MoveEndEvent(data)      => {Some(data.target)} 

            AttackEvent(_data)      => {None} // trying to attack a dead unit results in retargeting and a 0-tick attack requeue. Therefore stale attacks explicitly shouldn't be pruned.
            AbilityCastEvent(_data) => {None} // abilities with targeted effects manifest them as one of the above events so dont need to be handled directly.
            DeathEvent(_data)       => {None} // bogus return but never relevant so not worth changing to Option<EntityID>
            _DebugEvent(_data)      => {None}
        }
    }
}

