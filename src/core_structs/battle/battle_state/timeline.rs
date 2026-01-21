//! contains the EventTimeline struct - a sorted vector optimized for small event queues
use std::{cmp::PartialOrd, fmt::{Display, Write}};

use smallvec::SmallVec;

use crate::prelude::BattleEvent;

#[derive(PartialEq, Eq, Debug)]
pub struct EventTimeline {
    pub events: SmallVec<[EventContainer; 32]>,
    seq: i32
}

#[derive(PartialEq, Eq, Debug)]
pub struct EventContainer{
    pub event: BattleEvent,
    pub tick: u32,
    seq: i32
}

// Implement ordering that gives us min-heap on tick, max-heap on seq
impl Ord for EventContainer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Primary: lower tick comes first (min-heap)
        self.tick.cmp(&other.tick)
            // Secondary: higher seq comes first for LIFO (since seq increases)
            .then(other.seq.cmp(&self.seq))
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
            events: SmallVec::new(),
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

    pub fn push(&mut self, event: BattleEvent, tick: u32, current_tick: u32) {
        let cont = EventContainer {
            event,
            tick,
            seq: self.seq
        };
        self.seq += 1;

        if tick == current_tick {
            // scan back from end as this will likely go right near the end.
            let mut flag: usize = 0;

            for (i, val) in self.events.iter().enumerate().rev() {
                if val > &cont {
                    flag = i+1;
                    break
                }
            }
            self.events.insert(flag, cont);
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
            writeln!(buf, "Tick {}: {:?}", event.tick, event.event).unwrap();
        }
        write!(f, "{}\n", buf)
    }
}
