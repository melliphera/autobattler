use smallvec::SmallVec;
use CacheCleanliness::*;

use crate::prelude::*;

#[derive(Debug)]
pub enum CacheCleanliness {
    Clean,             // cache is cleaned whenever the team's position has been queried.
    DirtyAfter(Tick),   // any active movement invalidates the cache every tick. Clean during tick, dirty after.
    Dirty              // any unit dying invalidates the cache instantly.
}


#[derive(Debug, Clone, Copy)]
pub struct LocationTag {pub id: EntityID, pub location: BattleSubtile}

#[derive(Debug)]
pub struct TeamPositionCache {
    data: SmallVec<[LocationTag; 16]>,
    cleanliness:  CacheCleanliness
}

impl TeamPositionCache {
    pub fn new() -> Self { Self {data: SmallVec::new(), cleanliness: Dirty}}

    pub fn get_positions(&self, tick: Tick) -> Option<&SmallVec<[LocationTag; 16]>> {
        match self.cleanliness {
            Clean => Some(&self.data),
            Dirty => None,
            DirtyAfter(clean_tick) => {
                if tick == clean_tick {
                    Some(&self.data)
                } else {
                    None
                }
            }
        }
    }

    pub fn acknowledge_death(&mut self) {
        self.cleanliness = Dirty
    }

    pub fn acknowledge_movement(&mut self, tick: Tick) {
        self.cleanliness = DirtyAfter(tick)
    }

    pub fn update(&mut self, data: SmallVec<[LocationTag; 16]>, cleanliness: CacheCleanliness) {
        self.data = data;
        self.cleanliness = cleanliness
    }
}

