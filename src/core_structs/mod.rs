pub(crate) mod unit;
pub(crate) mod battle;

pub use crate::core_structs::battle::battle_state::BattleState;

pub mod prelude {
    pub(crate) use crate::core_structs::unit::prelude::*;
    pub use crate::core_structs::battle::prelude::*;
}