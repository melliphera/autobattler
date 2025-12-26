//! Contains everything pertaining to the BattleState struct - the overarching struct which manages combat flow.
pub mod instantiation;
pub mod event_handling;
pub mod helper_methods;
pub mod operation;
pub mod entity_list;

use rand::rngs::StdRng;
use priority_queue::PriorityQueue;
use rustc_hash::FxHasher;
use std::{cmp::Reverse, hash::BuildHasherDefault};
use entity_list::EntityList;

use crate::core_structs::prelude::*;

pub struct BattleState {
    pub live_units: EntityList<25>, // all living units. dead units can be seperately handled in Godot.
    pub rng: StdRng, 
    timeline: PriorityQueue<BattleEvent, Reverse<u32>, BuildHasherDefault<FxHasher>>,
    pub events_called: i32
}

