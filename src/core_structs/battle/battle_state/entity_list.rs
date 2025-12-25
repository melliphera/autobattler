use crate::core_structs::prelude::*;

pub struct EntityList<const N: usize> {
    entities: [Option<BattleUnit>; N], // allows for eg 5 allies, 5 spawned constructs, 15 enemies at once.
    spawns_in_slot: [u16; N],           
    next_id: usize,
    lowest_nonempty: usize,
    pub is_full: bool
}

impl<const N: usize> EntityList<N> {
    pub fn new() -> Self {
        Self {
            entities: std::array::from_fn(|_| None),
            spawns_in_slot: [0; N],
            next_id: 0,
            lowest_nonempty: 0,
            is_full: false
        }
    }

    pub fn get(&self, entity_id: &EntityID) -> Option<&BattleUnit> {
        let eid_index = entity_id.0 as usize;
        if entity_id.1 == self.spawns_in_slot[eid_index] {
            self.entities[eid_index].as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, entity_id: &EntityID) -> Option<&mut BattleUnit> {
        let eid_index = entity_id.0 as usize;
        if entity_id.1 == self.spawns_in_slot[eid_index] {
            self.entities[eid_index].as_mut()
        } else {
            None
        }
    }

    pub fn spawn(&mut self, unit: BattleUnit) -> Result<(), ()> {
        if self.is_full {
            return Err(())
        }
        self.entities[self.next_id] = Some(unit);
        self.spawns_in_slot[self.next_id] += 1;
        if self.lowest_nonempty < N {
            self.next_id += 1;
            self.lowest_nonempty += 1; 
        } else {
            self.find_next_empty()
        }
        Ok(())
    }

    pub fn remove(&mut self, id: &EntityID) {
        if self.spawns_in_slot[id.0 as usize] == id.1 {
            self.entities[id.0 as usize] = None
        }
    }

    pub fn get_next_id(&self) -> EntityID {
        // returns an empty ID to spawn the next unit with.
        EntityID(self.next_id as u16, self.spawns_in_slot[self.next_id]+1)
    }

    fn find_next_empty(&mut self) {
        for counter in 0..N {
            let ind = (self.next_id + counter) % N;
            if self.entities[ind].is_none() {
                self.next_id = ind;
                return
            }
        }
        // only happens if for loop continues with no found empties.
        self.is_full = true
    }

    pub fn iter(&self) -> EntityIterator<'_, N> {
        EntityIterator { store: &self, index: 0 }
    }

    // Mutable iterator
    pub fn iter_mut(&mut self) -> EntityIterMut<'_, N> {
        EntityIterMut {
            store: self,
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }
}

impl<'a, const N: usize> IntoIterator for &'a EntityList<N> {
    type Item = &'a BattleUnit;
    type IntoIter = EntityIterator<'a, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct EntityIterator<'a, const N: usize> {
    store: &'a EntityList<N>,
    index: usize,
}

impl<'a, const N: usize> Iterator for EntityIterator<'a, N> {
    type Item = &'a BattleUnit;
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < N {
            let idx = self.index;
            self.index += 1;
            
            if let Some(unit) = &self.store.entities[idx] {
                return Some(unit);
            }
        }
        None
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.store.lowest_nonempty-self.index))
    }
}


// Mutable iterator
pub struct EntityIterMut<'a, const N: usize> {
    store: &'a mut EntityList<N>,
    index: usize,
}

impl<'a, const N: usize> Iterator for EntityIterMut<'a, N> {
    type Item = &'a mut BattleUnit;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Need unsafe for mutable iteration due to borrowing rules
        while self.index < N {
            let idx = self.index;
            self.index += 1;
            
            // SAFETY: We never return overlapping mutable references
            // Each call to next() moves index forward
            unsafe {
                let slot = self.store.entities.as_mut_ptr().add(idx);
                if let Some(unit) = &mut *slot {
                    return Some(unit);
                }
            }
        }
        None
    }
}

// Implement IntoIterator for &mut EntityStore
impl<'a, const N: usize> IntoIterator for &'a mut EntityList<N> {
    type Item = &'a mut BattleUnit;
    type IntoIter = EntityIterMut<'a, N>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

