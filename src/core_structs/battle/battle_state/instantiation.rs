use std::{cmp::Reverse, collections::HashMap};
use fixedstr::str32;
use priority_queue::PriorityQueue;
use rand::{SeedableRng, rngs::StdRng};

use crate::prelude::*;

impl BattleState {
    pub(crate) fn new() -> Self {
        let seed: u64 = rand::random();
        println!("Generating new BattleState with seed: {}", seed);
        BattleState::new_seeded(seed)
    }

    pub(crate) fn new_seeded(seed: u64) -> Self {
        BattleState { live_units: HashMap::new(), timeline: PriorityQueue::new(), next_id: EntityID(0), rng: StdRng::seed_from_u64(seed)}
    }

    fn queue_event(&mut self, event: BattleEvent, tick: u32) {
        self.timeline.push(event, Reverse(tick));
    }

    pub(crate) fn spawn_ally_from_id(&mut self, id: UnitTemplateID, position: BattlePosition) {
        let template = UNIT_DATABASE[id.0 as usize];
        let ability = get_ability(id);
        let b = BattleUnit { 
            id: self.next_id, 
            unit: id, 
            team: Team::Player,
            position: position.to_logical(),
            current_movement: None,
            target: None, 
            mana: Mana(0),
            max_mana: match ability {Some(a) => a.get_mana(), _=> Mana(0)},
            ability,
            shield: Shield(None), 
            incoming_damage_handlers: HashMap::new(), 
            outgoing_damage_handlers: HashMap::new(), 
            temp_stat_modifiers: HashMap::new(),
            range_squared: template.attack_range.to_squared(),
            
            current_hp: template.hitpoints, 
            max_hp: template.hitpoints,
            move_speed: template.move_speed,
            defence: template.defence, 
            magic_resist: template.magic_resist, 
            attack_type: template.attack_type, 
            attack: template.attack, 
            attack_delay: template.attack_delay, 
            crit_chance: template.crit_chance, 

        };
        self.live_units.insert(self.next_id, b);
        self.next_id = EntityID(self.next_id.0 + 1)
    }

    pub(crate) fn spawn_enemy_from_id(&mut self, id: UnitTemplateID, position: BattlePosition) {
        let template = match id.1 {
            Roster::Human => UNIT_DATABASE[id.0 as usize],
            Roster::NPC  => ENEMY_DATABASE[id.0 as usize]
        };
        let ability = get_ability(id);

        let b = BattleUnit {
            id: self.next_id, 
            unit: id, 
            team: Team::Opponent,
            position: position.to_logical(),
            current_movement: None,
            move_speed: template.move_speed,
            current_hp: template.hitpoints, 
            max_hp: template.hitpoints, 
            defence: template.defence, 
            magic_resist: template.magic_resist, 
            target: None,
            mana: Mana(0),
            max_mana: match ability {Some(a) => a.get_mana(), _=> Mana(0)},
            ability: ability,
            attack_type: template.attack_type, 
            attack: template.attack, 
            attack_delay: template.attack_delay, 
            range_squared: template.attack_range.to_squared(), 

            crit_chance: template.crit_chance, 
            shield: Shield(None), 
            incoming_damage_handlers: HashMap::new(), 
            outgoing_damage_handlers: HashMap::new(), 
            temp_stat_modifiers: HashMap::new()
        };
        self.live_units.insert(self.next_id, b);
        self.next_id = EntityID(self.next_id.0 + 1)
    }

    fn initialize(&mut self) {
        // load any start-of-fight state (not implemented)
        println!("Initialising battlefield.");
        let ally_positions = self.get_positions_by_team(Team::Player, 0);
        let opponent_positions = self.get_positions_by_team(Team::Opponent, 0);
        let mut initial_event_stack: Vec<BattleEvent> = Vec::with_capacity(self.live_units.len());

        let cloned_for_keys = self.live_units.clone(); // i'm not smart enough to know why this is needed, i just know it is.
        let keys = cloned_for_keys.keys();

        for key in keys {
            let unit = self.live_units.get_mut(key).unwrap();
            let target_positions = if unit.team == Team::Player {opponent_positions.clone()} else {ally_positions.clone()};
            unit.find_target(&target_positions);
            let battle_event = unit.attack_current_target();
            initial_event_stack.push(battle_event);
            
        }
        for event in initial_event_stack.iter() {
            self.queue_event(*event, 0);
        }
    }

    pub(crate) fn simulate(&mut self, max_tick: u32) {
        
        self.initialize();
        while let Some((event, tick)) = self.timeline.pop() { // while there are things in the timeline
            if tick.0 > max_tick { // max fight length.
                println!("Fight timed out!");
                return;
            }
            let to_queue = self.execute_event(event, tick);   // do the thing. 
            for (event, tick) in to_queue.into_iter() {       // add newly produced items ot the queue.
                self.queue_event(event, tick);
            } 
        }
    }

    fn execute_event(&mut self, event: BattleEvent, tick: Reverse<u32>) -> Vec<(BattleEvent, u32)> {
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
                // check if ability is targeted to current attack target. - if it is, ensure current target is valid.
                let source = self.live_units.get(&data.source).unwrap();
                if data.ability.target_paradigm == TargetParadigm::CurrentTarget && source.target.is_none() {
                    let opp_pos = &self.get_opponent_positions(data.source, tick.0);
                    let source = self.live_units.get_mut(&data.source).unwrap();
                    source.find_target(opp_pos);
                }
                { // set mana to 0
                    let source = self.live_units.get_mut(&data.source).unwrap();
                    source.mana = Mana(0)
                }
                // apply ability effect to each target
                new_events = data.ability.cast(data.source, &self, tick.0)
            }
            BuffEvent(data) => {
                let target_unit = self.live_units.get_mut(&data.target).unwrap();
                let buff_bin = match data.buff.buff_type {
                    BuffEffect::AttackDamageModifier(_)  => &mut target_unit.temp_stat_modifiers,
                    BuffEffect::FlatIncomingReduction(_) => &mut target_unit.incoming_damage_handlers
                }; 
                let _ = *buff_bin.entry((data.buff.id, data.source))      // find entry if it exists
                                .and_modify(|container| container.add_stack(data.buff.buff_type.get_magnitude()))   // and change it by adding 1 stack with the current magnitude.                 
                                .or_insert(BuffContainer::new_from(data.buff)); // or create one with value 1.
            }
            MoveEvent(data) => {
                // set unit in motion
                let target_unit = self.live_units.get_mut(&data.target).unwrap(); // event would be flushed if target was daed
                target_unit.current_movement = Some(data);

                println!("tick {}:  \t{} is moving to ({}, {}) over {} ticks", tick.0, target_unit.unit.get_name(), data.end_pos.x/LOGICAL_SUBTILES, data.end_pos.y/LOGICAL_SUBTILES, data.end_tick-data.start_tick);

                // queue movement end event
                new_events.push((MoveEndEvent(MoveEndData {
                    target: data.target,
                    end_pos: data.end_pos
                }), data.end_tick));
            }
            MoveEndEvent(data) => {
                let mut move_again = false; // if new move event should be instantly triggered
                {
                // update unit position variables
                let target_unit = self.live_units.get_mut(&data.target).unwrap(); // event would be flushed if target was daed
                target_unit.current_movement = None;
                target_unit.position = data.end_pos;
                println!("tick {}:  \t{} has arrived at ({}, {})", tick.0, target_unit.unit.get_name(), data.end_pos.x/LOGICAL_SUBTILES, data.end_pos.y/LOGICAL_SUBTILES);
                }
                // double grab so needs immutable 
                // check if in range to requeue move event if necessary.
                let attacker = self.live_units.get(&data.target).unwrap();
                if let Some(unit) = attacker.target {
                    let target_distance = attacker.position.distance_squared_to(&self.live_units.get(&unit).unwrap().position);
                    if target_distance > attacker.range_squared {
                        move_again = true
                    }
                }
                if move_again {
                    new_events.push((MoveEvent(attacker.path(&self, tick.0)), tick.0));
                    return new_events;
                }
            }
                
            _ => {unreachable!("Other events not implemented yet.")}
        };

        // clear out any dead, and remove their events from the timeline.
        for id in new_dead.iter() {
            println!("Unit {}: {} has died!", id.0, self.get_name(*id));
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
            println!("Added cast: {} - {} to the queue.", source.unit.get_name(), source.ability.expect("trying to cast None ability.").name);
            return new_events
        }

        // 3 - attack target (or move into range if currently out of range)
        let next_attack_tick = tick.0 + source.attack_delay.0 as u32;
        new_events.push((source.attack_current_target(), next_attack_tick));
        new_events
    }
}