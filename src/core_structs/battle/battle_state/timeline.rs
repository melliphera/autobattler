//! contains the EventTimeline struct - a sorted vector optimized for small event queues
use std::{cmp::PartialOrd, fmt::{Display, Write}};

use smallvec::SmallVec;

use crate::prelude::*;

#[derive(PartialEq, Eq, Debug)]
pub struct EventTimeline {
    //pub events: Vec<EventContainer>,
    pub events: SmallVec<[EventContainer; 32]>,
    seq: i32
}

#[derive(PartialEq, Eq, Debug)]
pub struct EventContainer{
    pub event: BattleEvent,
    pub tick: Tick,
    seq: i32
}

// Implement ordering that gives us FIFO behavior for events with the same tick.
impl Ord for EventContainer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Primary: lower tick comes first (min-heap)
        self.tick.cmp(&other.tick)
            // Secondary: lower seq comes first for FIFO (since seq increases)
            .then(self.seq.cmp(&other.seq))
    }
}

impl PartialOrd for EventContainer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl EventTimeline {
    pub fn new() -> Self {
        Self {
            events: SmallVec::with_capacity(32),
            seq: 0
        }
    }

    #[cfg(test)]
    pub fn iter(&self) -> impl Iterator<Item=&EventContainer> {
        self.events.iter()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn push(&mut self, event: BattleEvent, tick: Tick, current_tick: Tick) {
        // create event container
        let cont = EventContainer {
            event,
            tick,
            seq: self.seq
        };
        self.seq += 1;

        if tick == current_tick {
            // whenever same-tick events are queued, they should be processed immediately, so push to end. (Events are called with self.pop()).
            self.events.push(cont);

        } else {
            // scan from front
            let mut flag: usize = self.events.len();

            for (i, val) in self.events.iter().enumerate() {
                if val < &cont {
                    flag = i;
                    break
                }
            }
            self.events.insert(flag, cont);
        }
    }

    pub fn pop(&mut self) -> Option<EventContainer> {
        self.events.pop()
    }

    pub fn retain(&mut self, mut f: impl FnMut(&EventContainer) -> bool) {
        self.events.retain(|e| f(e));
    }
}

impl Display for EventTimeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::with_capacity(self.len() * 10);
        for event in self.events.iter() {
            writeln!(buf, "Tick {}: {:?}", event.tick.0, event.event).unwrap();
        }
        write!(f, "{}\n", buf)
    }
}
