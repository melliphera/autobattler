//! Contains everything pertaining to the BattleState struct - the overarching struct which manages combat flow.
pub mod instantiation;
pub mod event_handling;
pub mod helper_methods;
pub mod operation;
pub mod entity_list;
pub mod timeline;
pub mod blocked_arena;
pub mod godot_interface;

use rand::rngs::StdRng;
use entity_list::EntityList;
use blocked_arena::BlockedArena;

use crate::core_structs::battle::battle_state::{godot_interface::godot_events::GodotEvent, timeline::EventTimeline};

#[derive(Debug)]
pub struct BattleState {
    pub live_units: EntityList<25>, // all living units. dead units can be seperately handled in Godot. <N> represents max number concurrently alive.
    pub rng: StdRng, 
    timeline: EventTimeline,
    last_processed_tick: u32,
    pub blocked: BlockedArena,
    pub godot_event_buffer: Option<Vec<GodotEvent>>, // if Some, all processed events get converted to GodotEvent and stored.

    pub events_called: i32

}

