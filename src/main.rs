pub(crate) mod core_structs;

use crate::core_structs::{battle::battle_state::BattleState, unit::prelude::*};
use Roster::*;

pub(crate) mod prelude {
    pub(crate) use crate::core_structs::unit::prelude::*;
    pub(crate) use crate::core_structs::battle::prelude::*;
}

pub const MAX_FIGHT_LENGTH: u32 = 30000;

fn main() {
    let mut b = BattleState::new();
    b.spawn_ally_from_id(UnitTemplateID(0, Human), BattlePosition { x: 4, y: 5 });
    b.spawn_enemy_from_id(UnitTemplateID(0, NPC),  BattlePosition { x: 9, y: 5 });
    b.spawn_enemy_from_id(UnitTemplateID(1, NPC),  BattlePosition { x: 4, y: 4 });
    b.simulate(MAX_FIGHT_LENGTH);
}
