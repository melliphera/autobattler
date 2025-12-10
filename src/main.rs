pub mod core_structs;

use crate::core_structs::{battle::battle_state::BattleState, unit::prelude::*};
use Roster::*;

fn main() {
    let mut b = BattleState::new();
    b.spawn_ally_from_id(UnitTemplateID(0, Human), BattlePosition { x: 4, y: 5 });
    b.spawn_enemy_from_id(UnitTemplateID(0, NPC),  BattlePosition { x: 5, y: 5 });
    b.spawn_enemy_from_id(UnitTemplateID(0, NPC),  BattlePosition { x: 5, y: 4 });
    b.simulate();
}
