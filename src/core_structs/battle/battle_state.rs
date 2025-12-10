use std::{cmp::Reverse, collections::HashMap};
use fixedstr::str32;
use priority_queue::PriorityQueue;

use crate::core_structs::{battle::battle_event::AttackData, unit::prelude::*};
use super::battle_event::BattleEvent::{self, *};


pub struct BattleState {
    pub live_units: HashMap<EntityID, BattleUnit>, // all living units. dead units can be seperately handled in Godot.
    timeline: PriorityQueue<BattleEvent, Reverse<u32>>
}

impl BattleState {
    pub fn new() -> Self {
        return BattleState { live_units: HashMap::new(), timeline: PriorityQueue::new() }
    }

    fn queue_event(&mut self, event: BattleEvent, tick: u32) {
        self.timeline.push(event, Reverse(tick));
    }

    fn get_name(&self, id: EntityID) -> str32 {
        self.live_units.get(&id).unwrap().unit.get_name()
    }

    pub fn spawn_ally_from_id(&mut self, id: UnitTemplateID, position: BattlePosition) {
        let entity_id = EntityID(self.live_units.len() as u8);
        let template = UNIT_DATABASE[id.0 as usize];
        let b = BattleUnit { 
            id: entity_id, 
            unit: id, 
            team: Team::Player,
            position: position,
            current_hp: template.hitpoints, 
            max_hp: template.hitpoints, 
            defence: template.defence, 
            magic_resist: template.magic_resist, 
            target: None, 
            attack_type: template.attack_type, 
            attack: template.attack, 
            attack_delay: template.attack_delay, 
            range_squared: template.attack_range.to_squared(), 
            crit_chance: template.crit_chance, 
            shield: Shield(None), 
            incoming_damage_handlers: Vec::new(), 
            outgoing_damage_handlers: Vec::new(), 
            temp_stat_modifiers: Vec::new()
        };
        self.live_units.insert(entity_id, b);
    }

    pub fn spawn_enemy_from_id(&mut self, id: UnitTemplateID, position: BattlePosition) {
        let entity_id = EntityID(self.live_units.len() as u8);
        let template = match id.1 {
            Roster::Human => UNIT_DATABASE[id.0 as usize],
            Roster::NPC  => ENEMY_DATABASE[id.0 as usize]
        };
        let b = BattleUnit { 
            id: entity_id, 
            unit: id, 
            team: Team::Opponent,
            position: position,
            current_hp: template.hitpoints, 
            max_hp: template.hitpoints, 
            defence: template.defence, 
            magic_resist: template.magic_resist, 
            target: None, 
            attack_type: template.attack_type, 
            attack: template.attack, 
            attack_delay: template.attack_delay, 
            range_squared: template.attack_range.to_squared(), 
            crit_chance: template.crit_chance, 
            shield: Shield(None), 
            incoming_damage_handlers: Vec::new(), 
            outgoing_damage_handlers: Vec::new(), 
            temp_stat_modifiers: Vec::new()
        };
        self.live_units.insert(entity_id, b);
    }

    fn initialize(&mut self) {
        // load any start-of-fight state (not implemented)
        println!("Initialising battlefield.");
        let ally_positions = self.get_positions(Team::Player);
        let opponent_positions = self.get_positions(Team::Opponent);
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

    fn get_positions(&self, team: Team) -> Vec<(EntityID, BattlePosition)> {
        self.live_units.iter()
                        .filter(|unit| unit.1.team == team)
                        .map(|unit| (unit.1.id, unit.1.position))
                        .collect()
    }

    fn get_opponent_positions(&self, id: EntityID)  -> Vec<(EntityID, BattlePosition)> {
        let team = self.live_units.get(&id).unwrap().team;
        self.get_positions(team.opponent())

    }

    pub fn simulate(&mut self) {
        
        self.initialize();
        while let Some((event, tick)) = self.timeline.pop() {      // while there are things in the timeline
            if let Some((next, tick)) = self.execute_event(event, tick) { // do the thing. 
                self.queue_event(next, tick);
            } 
        }
    }

    fn execute_event(&mut self, event: BattleEvent, tick: Reverse<u32>) -> Option<(BattleEvent, u32)> {
        //! execute the current event. This is gonna get bulky.
         
        //below allocations are for combat logging. Pre-unders are to circumvent inaccurate linting.
        let mut _target_name = str32::new();
        let (mut _max_hp, mut _pre_hp, mut _rem_hp) = (Hitpoints(0), Hitpoints(0), Hitpoints(0));

        let mut new_dead: Vec<EntityID> = Vec::new(); //entities that have died from this Event
        let source_id = event.get_source_id();
        let opp_positions = self.get_opponent_positions(source_id);

        match event {
            AttackEvent(data) => {
                let mut _successful_hit: bool = false; 

                {   // handle target stuff - use different code block for source if necessary later.

                    if let Some(target) = self.live_units.get_mut(&data.target) {
                        _pre_hp = target.current_hp;               
                        if let Some(DeathEvent(id)) = target.take_damage(data) {
                            new_dead.push(id);
                        }
                        _target_name = target.unit.get_name();
                        _max_hp = target.max_hp;
                        _rem_hp = target.current_hp;
                        _successful_hit = true;
                    } else {
                        println!("Target dead, requeueing new attack on this tick.");
                        let source = self.live_units.get_mut(&source_id).unwrap();
                        if let Some(target) = source.find_target(&opp_positions) { // same attack but to other target
                            return Some((AttackEvent(AttackData{target, ..data}), tick.0))
                        } else {
                            println!("No more targets found!");
                            return None;
                        }
                    }
                }   
                {   // handle source stuff e.g adding mana.
                    let source = self.live_units.get(&source_id).unwrap();
                    let source_name = source.unit.get_name();
                    if _successful_hit {
                        print!("tick {}:\t{} struck {} for {} damage!\t", tick.0, source_name, _target_name, _pre_hp.0 - _rem_hp.0);
                        println!("Remaining health {}/{}", _rem_hp.0, _max_hp.0);
                    } 
                }
            }
            _ => {unreachable!("Other events not implemented yet.")}
        };

        // clear out any dead, and remove their events from the timeline.
        for id in new_dead.iter() {
            println!("Unit {}: {} has died!", id.0, self.get_name(*id));
            self.live_units.remove(id);

            self.timeline.retain(|event, _prio| event.get_source_id() != *id);
            self.timeline.retain(|event, _prio| event.get_target_id() != *id);
        };
        

        // all below: queue the attacker's next event.

        // grab attacker as mutable 
        let source = self.live_units.get_mut(&source_id).unwrap();

        // 1 - does source need a new target?        find one if so.
        if source.target == None {
            source.find_target(&opp_positions);
        }

        // 1.5 - if no target can be found, don't queue another event
        if source.target == None {
            return None
        }

        // 2 - does source have full non-zero mana?  cast if so.
        // unimplemented

        // 3 - is source in range of target?         move if not.
        

        // 4 - attack target
        let next_attack_tick = tick.0 + source.attack_delay.0 as u32;
        Some((source.attack_current_target(), next_attack_tick))
    
    }

}