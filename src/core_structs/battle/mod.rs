pub(crate) mod battle_event;
pub(crate) mod battle_state;

pub mod prelude {
    pub(crate) use super::battle_event::BattleEvent::{self, *};
    pub(crate) use super::battle_state::event_handling::AttackContext;
    pub use super::battle_event::data_types::*;
    pub use super::battle_state::BattleState;
}