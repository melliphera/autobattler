//! Contains everything pertaining to the BattleState struct - the overarching struct which manages combat flow.
pub mod instantiation;
pub mod event_handling;
pub mod helper_methods;
pub mod operation;
pub mod entity_list;
pub mod timeline;

use rand::rngs::StdRng;
use entity_list::EntityList;

use crate::core_structs::{battle::battle_state::timeline::EventTimeline};

pub struct BattleState {
    pub live_units: EntityList<25>, // all living units. dead units can be seperately handled in Godot.
    pub rng: StdRng, 
    timeline: EventTimeline,
    last_processed_tick: u32,

    pub events_called: i32

}

