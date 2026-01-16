use crate::core_structs::prelude::*;

pub struct EntityList<const N: usize> {
    entities: [Option<BattleUnit>; N], 
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
        if self.lowest_nonempty < N-1 {
            self.next_id += 1;
            self.lowest_nonempty += 1; 
        } else {
            self.find_next_empty()
        }
        Ok(())
    }

    pub fn remove(&mut self, id: &EntityID) {
        if self.spawns_in_slot[id.0 as usize] == id.1 {
            self.entities[id.0 as usize] = None;
            self.is_full = false
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

// Implement IntoIterator for &mut EntityList
impl<'a, const N: usize> IntoIterator for &'a mut EntityList<N> {
    type Item = &'a mut BattleUnit;
    type IntoIter = EntityIterMut<'a, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::cell::RefCell;

    // Helper function to create a minimal BattleUnit for testing
    fn create_test_unit(id: EntityID) -> BattleUnit {
        BattleUnit {
            id,
            template: UnitTemplateID(0, Roster::Human),
            team: Team::Player,
            position: GridPosition { x: 0, y: 0 },
            last_position: GridPosition { x: 0, y: 0 },
            move_speed: MoveSpeed(1),
            current_movement: None,
            blocked_on_last_move: RefCell::new(None),
            current_hp: Hitpoints(100),
            max_hp: Hitpoints(100),
            defence: Mitigation(0),
            magic_resist: Mitigation(0),
            target: None,
            attack_type: DamageType::Physical,
            attack: AttackDamage(10),
            attack_delay: AttackTickDelay(60),
            range_squared: SquaredLogicalRange(1),
            crit_chance: CritChance(0),
            ability: None,
            mana: Mana(0),
            max_mana: Mana(0),
            shield: Shield(None),
            incoming_damage_handlers: HashMap::new(),
            outgoing_damage_handlers: HashMap::new(),
            temp_stat_modifiers: HashMap::new(),
        }
    }

    #[test]
    fn test_iter_returns_all_units_no_none_values() {
        // Create an EntityList with capacity for 5 units
        let mut entity_list: EntityList<5> = EntityList::new();

        // Spawn 3 units
        let id1 = entity_list.get_next_id();
        let unit1 = create_test_unit(id1);
        entity_list.spawn(unit1).expect("Failed to spawn unit 1");

        let id2 = entity_list.get_next_id();
        let unit2 = create_test_unit(id2);
        entity_list.spawn(unit2).expect("Failed to spawn unit 2");

        let id3 = entity_list.get_next_id();
        let unit3 = create_test_unit(id3);
        entity_list.spawn(unit3).expect("Failed to spawn unit 3");

        // Collect all units from the iterator
        let collected: Vec<&BattleUnit> = entity_list.iter().collect();

        // Verify we got exactly 3 units
        assert_eq!(collected.len(), 3, "Iterator should return exactly 3 units");

        // Verify all returned items are actual units with correct IDs
        assert_eq!(collected[0].id, id1, "First unit should have id1");
        assert_eq!(collected[1].id, id2, "Second unit should have id2");
        assert_eq!(collected[2].id, id3, "Third unit should have id3");

        // Verify count() works correctly
        assert_eq!(entity_list.iter().count(), 3, "count() should return 3");

        // Verify all units can be accessed without panicking
        for unit in entity_list.iter() {
            assert!(unit.max_hp.0 > 0, "Each unit should have valid max_hp");
        }
    }

    #[test]
    fn test_iter_with_gaps_skips_none_values() {
        // Create an EntityList and spawn units with gaps
        let mut entity_list: EntityList<6> = EntityList::new();

        // Spawn 4 units
        let id1 = entity_list.get_next_id();
        entity_list.spawn(create_test_unit(id1)).unwrap();

        let id2 = entity_list.get_next_id();
        entity_list.spawn(create_test_unit(id2)).unwrap();

        let id3 = entity_list.get_next_id();
        entity_list.spawn(create_test_unit(id3)).unwrap();

        let id4 = entity_list.get_next_id();
        entity_list.spawn(create_test_unit(id4)).unwrap();

        // Remove the second unit to create a gap
        entity_list.remove(&id2);

        // Collect units
        let collected: Vec<&BattleUnit> = entity_list.iter().collect();

        // Should have 3 units (4 spawned - 1 removed)
        assert_eq!(collected.len(), 3, "Iterator should return 3 units after one removal");

        // Verify the remaining units have the correct IDs (should skip the removed one)
        assert_eq!(collected[0].id, id1);
        assert_eq!(collected[1].id, id3);
        assert_eq!(collected[2].id, id4);

        // Verify no None values were returned (implicit by the fact we got BattleUnit references)
        for unit in entity_list.iter() {
            // If we got here, we have a valid unit reference (not None)
            assert_ne!(unit.id.0, u16::MAX, "Should be a valid unit");
        }
    }

    #[test]
    fn test_iter_empty_list() {
        let entity_list: EntityList<5> = EntityList::new();

        // Iterator should return no items
        let collected: Vec<&BattleUnit> = entity_list.iter().collect();
        assert_eq!(collected.len(), 0, "Empty list should yield no items");

        // count() should be 0
        assert_eq!(entity_list.iter().count(), 0);
    }

    #[test]
    fn test_iter_full_list() {
        const SIZE: usize = 3;
        let mut entity_list: EntityList<SIZE> = EntityList::new();

        // Fill the list completely
        let mut ids = Vec::new();
        for _ in 0..SIZE {
            let id = entity_list.get_next_id();
            ids.push(id);
            entity_list.spawn(create_test_unit(id)).unwrap();
        }

        // Collect all units
        let collected: Vec<&BattleUnit> = entity_list.iter().collect();
        assert_eq!(collected.len(), SIZE, "Full list should return all units");

        // Verify all IDs match
        for (i, unit) in collected.iter().enumerate() {
            assert_eq!(unit.id, ids[i], "Unit ID should match at index {}", i);
        }

        // Verify attempting to add another unit fails (list is full)
        let result = entity_list.spawn(create_test_unit(entity_list.get_next_id()));
        assert!(result.is_err(), "Should not be able to spawn in a full list");
        assert!(entity_list.is_full, "is_full should be true after spawn fails");
    }

    #[test]
    fn test_iter_works_as_collection_iterator() {
        let mut entity_list: EntityList<4> = EntityList::new();

        // Add some units
        entity_list.spawn(create_test_unit(entity_list.get_next_id())).unwrap();
        entity_list.spawn(create_test_unit(entity_list.get_next_id())).unwrap();

        // Test that standard iterator methods work
        assert_eq!(entity_list.iter().count(), 2);

        // Test nth()
        assert!(entity_list.iter().nth(0).is_some());
        assert!(entity_list.iter().nth(1).is_some());
        assert!(entity_list.iter().nth(2).is_none());

        // Test for_each
        let mut count = 0;
        entity_list.iter().for_each(|_| count += 1);
        assert_eq!(count, 2);

        // Test any()
        assert!(entity_list.iter().any(|u| u.team == Team::Player));

        // Test all()
        assert!(entity_list.iter().all(|u| u.max_hp.0 > 0));

        // Test filter and count
        let player_count = entity_list.iter()
            .filter(|u| u.team == Team::Player)
            .count();
        assert_eq!(player_count, 2);
    }
}

