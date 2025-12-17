//! Contains everything pertaining to the BattleState struct - the overarching struct which manages combat flow.
pub mod instantiation;
pub mod event_processing;
pub mod helper_methods;

use std::collections::HashMap;
use rand::rngs::StdRng;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;

use crate::prelude::*;

pub(crate) struct BattleState {
    pub(crate) live_units: HashMap<EntityID, BattleUnit>, // all living units. dead units can be seperately handled in Godot.
    pub(crate) rng: StdRng, 
    timeline: PriorityQueue<BattleEvent, Reverse<u32>>,
    next_id: EntityID //id to instantiate next unit with
}