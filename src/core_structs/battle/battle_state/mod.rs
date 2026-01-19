//! Contains everything pertaining to the BattleState struct - the overarching struct which manages combat flow.
pub mod instantiation;
pub mod event_handling;
pub mod helper_methods;
pub mod operation;
pub mod entity_list;
pub mod timeline;
pub mod blocked_arena;
pub mod godot_interface;

use entity_list::EntityList;
use blocked_arena::BlockedArena;
use rand_chacha::ChaCha8Rng;

use crate::{core_structs::battle::battle_state::{godot_interface::godot_events::GodotEvent, timeline::EventTimeline}, prelude::EntityID};

pub struct BattleState {
    pub live_units: EntityList<25>, // all living units. dead units can be seperately handled in Godot. <N> represents max number concurrently alive.
    pub rng: ChaCha8Rng, 
    timeline: EventTimeline,
    last_processed_tick: u32,
    pub blocked: BlockedArena,
    pub godot_event_buffer: Option<Vec<GodotEvent>>, // if Some, all processed events get converted to GodotEvent and stored.
    pub unit_manifest: Option<Vec<(EntityID, fixedstr::str32)>>,

    pub events_called: i32

}

