use crate::core_structs::prelude::*;
use std::cmp::Reverse;
use fixedstr::str32;

impl BattleState {
    pub(super) fn queue_event(&mut self, event: BattleEvent, tick: u32) {
        self.timeline.push(event, Reverse(tick));
    }

    pub(crate) fn simulate(&mut self, max_tick: u32) -> Option<Team> {
        
        self.initialize();
        while let Some((event, tick)) = self.timeline.pop() { // while there are things in the timeline
            if tick.0 > max_tick { // max fight length.
                //println!("Fight timed out!");
                return None;
            }
            let to_queue = self.execute_event(event, tick);   // do the thing. 
            for (event, tick) in to_queue.into_iter() {       // add newly produced items ot the queue.
                self.queue_event(event, tick);
            } 
        }
        Some(self.live_units.iter().next().unwrap().team)        // Events should only run dry when one team is fully dead.
    }

    pub(super) fn execute_event(&mut self, event: BattleEvent, tick: Reverse<u32>) -> Vec<(BattleEvent, u32)> {
        //! execute the current event. This is gonna get bulky.
         
        // below allocations are for combat logging. Pre-unders are to circumvent inaccurate linting.
        let mut _target_name = str32::new();

        let mut new_dead: Vec<EntityID> = Vec::new(); // entities that have died from this Event
        let mut new_events: Vec<(BattleEvent, u32)> = Vec::new();

        match event {
            AttackEvent(_data) => {
                let attack_ctx = AttackContext { event, tick };
                (new_events, new_dead) = self.process_attack_event(attack_ctx); // overwrite safe because original vecs are definitely empty.
                
                //if the unit is moving instead of attacking, dont process a new attack.
                if let Some((MoveEvent(_data), _tick)) = new_events.iter().next() {
                    return new_events
                }

            }
            AbilityCastEvent(data) => {
                new_events = self.process_ability_cast(data, tick.0)
            }
            BuffEvent(data) => {
                self.process_buff_event(data); // doesn't inherently spawn new events.
            }
            MoveEvent(data) => {
                new_events = self.process_move_event(data, tick.0); // spawns a MoveEndEvent
            }
            MoveEndEvent(data) => {
                // if move_end processing returns an event, its another Move. therefore do not attack so return immediately.
                if let Some(move_event) = self.process_move_end_event(data, tick.0) {
                    return vec![(move_event, tick.0)];
                }
            }
            RawDamageEvent(data) => {
                let target = self.live_units.get_mut(&data.target).unwrap();
                if let Some(DeathEvent(id)) = target.take_damage(data) {
                    new_dead.push(id);
                }
            }
            HealEvent(data) => {
                let target = self.live_units.get_mut(&data.target).unwrap();
                target.heal(data)
            }
            ShieldEvent(data) => {
                let target = self.live_units.get_mut(&data.target).unwrap();
                target.shield(data)
            }
            DeathEvent(_) => {
                unreachable!("DeathEvent can't be called to the timeline.")
            }
            _DebugEvent(payload) =>  {
                self.process_debug_event(payload);
                self.queue_event(_DebugEvent(payload), tick.0+payload.delay);
            }
        };

        // clear out any dead, and remove their events from the timeline.
        for id in new_dead.iter() {
            //println!("Unit {}: {} has died!", id.0, self.get_name(*id));
            self.live_units.remove(id);
            self.timeline.retain(|event, _prio| event.get_source_id() != Some(*id) && event.get_target_id() != Some(*id));
        };
        
        // -- all below queue the attacker's next event --
        // therefore only relevant for AbilityCast, Attack or MoveEnd events
        match event {
            AbilityCastEvent(_) | AttackEvent(_) | MoveEndEvent(_) => {}
            _ => { return new_events }
        }

        // if source is dead, don't queue them another event.
        // THIS IS SCUFFED - IF MORE NEW EVENTS COME DOWN HERE, CHECK THEY RETURN SOME ON get_source_id()!!
        let source_id_opt = event.get_source_id();
        if let None = source_id_opt { 
            return new_events
        };

        // grab attacker as mutable 
        let source_id = source_id_opt.expect("Past a let None, this shouldn't ever show up.");
        let opp_positions = self.get_opponent_positions(source_id, tick.0);

        let source = self.live_units.get_mut(&source_id).unwrap();


        // 1 - does source need a new target? find one if so.
        if source.target == None {
            source.find_target(&opp_positions);
        }

        // 1.5 - if no target can be found, don't queue another event
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
                tick.0 + source.attack_delay.0 as u32
            ));
            //println!("Added cast: {} - {} to the queue.", source.template.get_name(), source.ability.expect("trying to cast None ability.").name);
            return new_events
        }

        // 3 - attack target (or move into range if currently out of range)
        let next_attack_tick = tick.0 + source.attack_delay.0 as u32;
        new_events.push((source.attack_current_target(), next_attack_tick));
        new_events
    }
}