//! Outward facing impls for BattleState, to communicate with Godot

//#[allow(dead_code)]

pub mod core_structs;
use core_structs::prelude::*;

pub mod dev_tools;
pub use core_structs::prelude;

#[cfg(test)]
pub mod tests;

pub const MAX_FIGHT_LENGTH: u32 = 30000;

#[cfg(test)]
pub mod tests_local {
    use super::*;
    use Roster::*;

    #[test]
    fn it_works() {
        let human_team = [
            (0, (5, 3)), // knight
            (1, (1, 3)), // ranger
            (2, (1, 2))  // mage
        ];

        let npc_team = [
            (0, (9, 1)),
            (0, (9, 3)),
            (0, (8, 0)),
            (0, (8, 1)),
            (0, (8, 2)),
            (0, (8, 3)),
            (0, (8, 4)),
        ];

        let mut b = BattleState::new_seeded(9405163005650660990)
                        //.with_debug(prelude::DebugInfo::_EventQueue, 500)
                        //.with_debug(prelude::DebugInfo::_HealthAndPos, 500)
                        ;

        for (id, (x, y)) in human_team.iter() {
            _ = b.spawn_ally_from_id(UnitTemplateID(*id, Human), GridPosition  { x: *x, y: *y });
        }

        for (id, (x, y)) in npc_team.iter() {
            _ = b.spawn_enemy_from_id(UnitTemplateID(*id, NPC),  GridPosition { x: *x, y: *y });
        }

        b.simulate(MAX_FIGHT_LENGTH);
    }
    
    #[test]
    fn tile_vs_logical_distance_consistency() {
        let a = GridPosition { x: 0, y: 0 }.to_logical();
        let b = GridPosition { x: 1, y: 0 }.to_logical();
        assert_eq!(a.distance_squared_to(&b), AttackRange(1).to_squared());
    } 

}
