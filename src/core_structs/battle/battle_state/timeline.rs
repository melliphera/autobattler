//! contains the EventTimeline struct, which implements a Binary Heap 
use std::{cmp::PartialOrd, fmt::Display};

use smallvec::SmallVec;

use crate::prelude::BattleEvent::{self, *};

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
            events: SmallVec::new(),
            seq: 0
        }
    }

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

        #[cfg(test)] {
            println!("Inserting event with tick: {}, seq: {} into list with\nticks: {:?}", cont.tick, cont.seq, self.events.iter().map(|e| e.tick).collect::<Vec<_>>());
            println!("Current tick: {}", current_tick);
        }

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

        #[cfg(test)] {
            println!("new ticks: {:?}", self.events.iter().map(|e| e.tick).collect::<Vec<_>>());
            assert!(self.events.windows(2).all(|w| w[0] >= w[1]),
            "Events aren't sorted in descending order!");
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
            buf += &match event.event {
                AttackEvent(_) => format!("Tick {}: AttackEvent\n", event.tick),
                DeathEvent(_) => format!("Tick {}: DeathEvent\n", event.tick),
                HealEvent(_) => format!("Tick {}: HealEvent\n", event.tick),
                AbilityCastEvent(_) => format!("Tick {}: AbilityCastEvent\n", event.tick),
                RawDamageEvent(_) => format!("Tick {}: RawDamageEvent\n", event.tick),
                BuffEvent(_) => format!("Tick {}: BuffEvent\n", event.tick),
                ShieldEvent(_) => format!("Tick {}: ShieldEvent\n", event.tick),
                MoveEvent(_) => format!("Tick {}: MoveEvent\n", event.tick),
                MoveEndEvent(_) => format!("Tick {}: MoveEndEvent\n", event.tick),
                _DebugEvent(_) => format!("Tick {}: DebugEvent\n", event.tick),
                // Add other BattleEvent variants as needed
            }
        }
        write!(f, "{}\n", buf)
    }
}