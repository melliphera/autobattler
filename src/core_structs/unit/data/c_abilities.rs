use crate::core_structs::unit::{abilities::AbilityPayload::*, prelude::*};
use crate::core_structs::unit::buffs_debuffs::BuffEffect::*;
use crate::core_structs::unit::abilities::TargetTeam;
use Roster::*;

use fixedstr::str32;

pub(crate) const fn get_ability(id: UnitTemplateID) -> Option<Ability> {
    match id {
        UnitTemplateID(0, Human) => { // Knight
            Some(Ability {
                name: str32::const_make("Bulwark"),
                mana_cost: 90,
                effects: [Some(BuffPayload( 
                            Buff{
                                id: BuffID(0),
                                buff_type: FlatIncomingReduction(Hitpoints(3)),
                                counter: None,    // permanent
                                max_stacks: None, // infinite scaling
                                priority: 0
                            }
                )), None, None, None],
                target: TargetTeam::Ally, // skips target-seeking logic.
                target_paradigm: TargetParadigm::Me, // selfcast
                cast_delay: None
            })
        },
        UnitTemplateID(1, Human) => { // Ranger
            Some(Ability {
                name: str32::const_make("Snipe"),
                mana_cost: 30,
                effects: [Some(Attack(Hitpoints(15), Physical)), None, None, None],
                target: TargetTeam::Enemy, 
                target_paradigm: TargetParadigm::Furthest(1), // furthest enemy
                cast_delay: None
            })
        },
        UnitTemplateID(2, Human) => { // Mage
            Some(Ability {
                name: str32::const_make("Chain Lightning"),
                mana_cost: 30,
                effects: [Some(Attack(Hitpoints(15), Magic)), None, None, None],
                target: TargetTeam::Enemy, 
                target_paradigm: TargetParadigm::Nearest(3), // furthest enemy
                cast_delay: None
            })
        },            
        _ => None
    }
}