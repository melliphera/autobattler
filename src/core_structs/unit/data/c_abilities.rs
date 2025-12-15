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
                effect: BuffPayload( 
                            Buff{
                                id: BuffID(0),
                                buff_type: FlatIncomingReduction(Hitpoints(3)),
                                counter: None,    // permanent
                                max_stacks: None, // infinite scaling
                                priority: 0
                            }
                ),
                target: TargetTeam::Ally, // skips target-seeking logic.
                target_paradigm: TargetParadigm::Me, // selfcast
                cast_delay: None
            })
        },
        _ => None

    }
}