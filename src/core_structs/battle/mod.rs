pub(crate) mod battle_event;
pub(crate) mod battle_state;

pub(crate) mod prelude {
    pub(crate) use super::battle_event::BattleEvent::{self, *};
    pub(crate) use super::battle_event::data_types::*;
    pub(crate) use super::battle_state::BattleState;
    pub(crate) use super::battle_state::event_processing::AttackContext;
}