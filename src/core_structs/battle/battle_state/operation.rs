use crate::core_structs::prelude::*;
use crate::core_structs::battle::battle_state::godot_interface::godot_events::GodotEvent;
use fixedstr::str32;

impl BattleState {
    pub(super) fn queue_event(&mut self, event: BattleEvent, tick: u32) {
        self.timeline.push(event, tick, self.last_processed_tick);
    }

    pub fn simulate(&mut self, max_tick: u32) -> (i32, Option<Team>) {
        
        self.initialize();
        while let Some(container) = self.timeline.pop() { // while there are things in the timeline
            let (tick, event) = (container.tick, container.event);
            if tick > max_tick { // max fight length.
                println!("Fight timed out!");
                print!("{:?}", self);
                return (self.events_called, None);
            }
            #[cfg(test)]
            println!("{}", self.timeline);

            let to_queue = self.execute_event(event, tick);   // do the thing. 
            for (event, tick) in to_queue.into_iter() {       // add newly produced items to the queue.
                self.queue_event(event, tick);
                self.events_called += 1
            } 
        }
        (self.events_called, Some(self.live_units.iter().next().map_or(Team::Player, |unit| unit.team)))    // Events should only run dry when one team is fully dead. If both teams are, player biased.
    }

    pub(super) fn execute_event(&mut self, event: BattleEvent, tick: u32) -> Vec<(BattleEvent, u32)> {
        //! execute the current event. This is gonna get bulky.

        #[cfg(test)] {
            // debug assertion that events are correctly executing in chronological order
            assert!(tick >= self.last_processed_tick, "Ticks executed non-chronologically!\nLast processed: {}\nCurrent event tick: {}", self.last_processed_tick, tick);

            // debug assertion that when there are no 0-tick events queued, each unit has exactly one AttackEvent, AbilityCastEvent or MoveEndEvent queued.
            // also sanity checks that the event is in a reasonable timeframe.
            if tick > self.last_processed_tick {
                let has_player = self.live_units.iter().any(|u| u.team == Team::Player);
                let has_opponent = self.live_units.iter().any(|u| u.team == Team::Opponent);
                if !(has_player && has_opponent) {
                    println!("One team is dead, skipping event queue sanity check.");
                } else {
                    println!("Checking event queue sanity at tick {}.", tick);

                    for unit in self.live_units.iter() {
                        if unit.id == event.get_source_id().expect("this shouldnt fail.") {
                            continue; // skip the unit that just acted.
                        }
                        let source_id = unit.id;
                        let mut event_count = 0;
                        for container in self.timeline.iter() {
                            let ev = &container.event;
                            if let Some(ev_source) = ev.get_source_for_pruning() {
                                if ev_source == source_id {
                                    match ev {
                                        AttackEvent(_) | AbilityCastEvent(_) | MoveEndEvent(_) => {
                                            event_count += 1;
                                            assert!(container.tick <= tick+300, "Unit {}'s next key event isn't for another {} ticks!'!", source_id.0, container.tick-tick);
                                        }
                                        _ => { println!("Event {:?} from unit {} is not a key event, ignoring.", ev, source_id.0)}
                                    }
                                }
                            }
                        }
                        assert!(event_count == 1, "Unit {} has {} queued events when they should have exactly one!\n\nUnit data:\n{:#?}", source_id.0, event_count, self.live_units.get(&source_id).unwrap());
                    }
                }
            }
        }

        // acknowledge current tick as "last processed tick" so spawning events on the same tick is more efficient.
        self.last_processed_tick = tick;

        // below allocations are for combat logging. Pre-unders are to circumvent inaccurate linting.
        let mut _target_name = str32::new();

        let mut new_dead: Vec<EntityID> = Vec::new(); // entities that have died from this Event
        let mut new_events: Vec<(BattleEvent, u32)> = Vec::new();

        match event {
            AttackEvent(_data) => {
                let attack_ctx = AttackContext { event, tick };
                (new_events, new_dead) = self.process_attack_event(attack_ctx); // overwrite safe because original vecs are definitely empty.
                
                //if the unit is moving, or the processed attack was stale, 
                if let Some(_container) = new_events.iter().next() {
                    return new_events
                }

            }
            AbilityCastEvent(data) => {
                new_events = self.process_ability_cast(data, tick)
            }
            BuffEvent(data) => {
                self.process_buff_event(data); // doesn't inherently spawn new events.
            }
            MoveEvent(data) => {
                new_events = self.process_move_event(data); // spawns a MoveEndEvent                
            }
            MoveEndEvent(data) => {
                // if move_end processing returns an event, its another MoveEvent. therefore do not attack so return immediately.
                if let Some(move_event) = self.process_move_end_event(data, tick) {
                    return vec![(move_event, tick)];
                }
            }
            RawDamageEvent(data) => {
                let target = self.live_units.get_mut(&data.target).unwrap();
                let (damage, dead_opt) = target.take_damage(data);

                if let Some(ref mut vec) = self.godot_event_buffer {
                    vec.push(GodotEvent::DamageTaken(data.target, damage));
                }

                if let Some(DeathEvent(id)) = dead_opt {
                    if let Some(ref mut vec) = self.godot_event_buffer {
                        vec.push(GodotEvent::Death(id));
                    }
                    new_dead.push(id);
                } 

            }
            HealEvent(data) => {
                let target = self.live_units.get_mut(&data.target).unwrap();
                let actually_healed = target.heal(data);

                if let Some(ref mut vec) = self.godot_event_buffer {
                    vec.push(GodotEvent::DamageHealed(data.target, actually_healed))
                }
            }
            ShieldEvent(data) => {
                let target = self.live_units.get_mut(&data.target).unwrap();
                if let Some(ref mut vec) = self.godot_event_buffer {
                    vec.push(GodotEvent::Shielded(data.target, data.amount))
                }
                target.shield(data)
            }
            DeathEvent(_) => {
                // note that DeathEvent logging is done at the site that death is registered, and the logic is handled by returning a DeathEvent. As such they are never fed to the timeline.
                // "the site that the death is registered" = unit.take_damage() calls which are triggered in the handling of AttackEvent and RawDamageEvent
                unreachable!("DeathEvent can't be called to the timeline.")
            }
            _DebugEvent(payload) =>  {
                self.process_debug_event(payload);
                self.queue_event(_DebugEvent(payload), tick+payload.delay);
            }
        };

        // clear out any dead, remove their events from the timeline, and unblock the tiles they are blocking.
        for id in new_dead.iter() {
            let unit = self.live_units.get(id).unwrap(); // this thing is about to die, it shouldnt be dead already.

            // clear movement/position data.
            if let Some(data) = unit.current_movement {
                self.blocked.set_coord(&data.end_pos, false);
            }
            self.blocked.set_coord(&unit.position, false);

            //println!("Unit {}: {} has died!", id.0, self.get_name(*id));
            // remove the unit and its events from the BattleState structure
            self.live_units.remove(id);
            self.timeline.retain(|container| 
                container.event.get_source_for_pruning() != Some(*id) && 
                container.event.get_target_for_pruning() != Some(*id));
        };
        
        // -- all below queue the attacker's next event --
        // therefore only relevant for AbilityCast, Attack or MoveEnd events
        match event {
            AbilityCastEvent(_) | AttackEvent(_) | MoveEndEvent(_) => {}
            _ => { return new_events }
        }

        // if source is dead, don't queue them another event.
        // THIS IS SCUFFED - IF MORE NEW EVENTS COME DOWN HERE, CHECK THEY RETURN Some ON get_source_id()!!
        let source_id_opt = event.get_source_for_pruning();
        if let None = source_id_opt { 
            return new_events
        };

        // grab attacker as mutable 
        let source_id = source_id_opt.expect("Past a let None, this shouldn't ever show up.");
        let opp_positions = self.get_opponent_positions(source_id, tick);

        let source = self.live_units.get_mut(&source_id).unwrap();


        // 1 - does source need a new target? find one if so.
        if source.target == None {
            source.find_target(&opp_positions);
        }

        // 1.5 - if no target can be found, don't queue another event - the enemy team is dead and the fight is won.
        if source.target == None {
            return new_events;
        }

        // 2 - does source have full non-zero mana?  cast if so.
        if source.mana == source.max_mana && source.mana != Mana(0) {
            new_events.push((
            AbilityCastEvent(
                AbilityData{
                    source: source_id,
                    ability: source.ability.expect("Trying to cast ability without having one.")
                }),
                tick + source.attack_delay.0 as u32
            ));
            //println!("Added cast: {} - {} to the queue.", source.template.get_name(), source.ability.expect("trying to cast None ability.").name);
            return new_events
        }

        // 3 - attack target (or move into range if currently out of range)
        let next_attack_tick = tick + source.attack_delay.0 as u32;
        new_events.push((source.attack_current_target(), next_attack_tick));
        new_events
    }

    #[cfg(test)] #[allow(private_interfaces)]
    pub fn step_event(&mut self) -> Vec<(BattleEvent, u32)> {
        //! pops a single event in the timeline and returns the event it spawns.
        if let Some(container) = self.timeline.pop() {
            let (tick, event) = (container.tick, container.event);

            println!("{}", self.timeline);

            self.execute_event(event, tick)   // do the thing. 

        } else {
            panic!("Test set up wrong, popped an empty event queue.")
        }
    }
}