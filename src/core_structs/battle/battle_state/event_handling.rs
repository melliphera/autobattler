//! Contains impl on BattleState for processing *specific* combat Events.
//! For larger during-fight operation, including execute_event() which calls these functions, see ./operation.rs
//! 
//! Likewise some event types are so simple to handle that they're directly handled in execute_event.
//! These types are currently RawDamageEvent, HealEvent and ShieldEvent.
use fixedstr::str32;

use crate::core_structs::prelude::*;


pub(crate) struct AttackContext {
    pub(super) event: BattleEvent,
    pub(super) tick: u32
}

impl BattleState {
    pub(super) fn process_attack_event(&mut self, ctx: AttackContext) -> (Vec<(BattleEvent, u32)>, Vec<EntityID>) {
        let (mut _max_hp, mut _pre_hp, mut _rem_hp) = (Hitpoints(0), Hitpoints(0), Hitpoints(0));
        let mut _target_name = str32::new();
        let mut new_dead = Vec::new();
        let mut new_events = Vec::new();

        let event = ctx.event;
        let mut data = match event {
            AttackEvent(d) => d,
            _ => unreachable!()
        };

        let mut _successful_hit: bool = false; 
        let source_id = event.get_source_id().unwrap(); // safe because it is Attack

        // grabbing some cheaply Copy data from source
        let source_cc: CritChance;
        let source_pos: BattleSubtile;
        let source_range: SquaredLogicalRange;
        let _source_name: str32;
        {
            let source_unit = self.live_units.get(&source_id).unwrap(); // safe because source is definitely still alive.
            source_cc = source_unit.crit_chance;
            source_pos = source_unit.get_position(ctx.tick);
            source_range = source_unit.range_squared;
            _source_name = source_unit.template.get_name()
        }

        {   
            // handle target stuff - use different code block for source if necessary later.
            if self.live_units.get(&data.target).is_none() {
                //println!("Target dead, requeueing new attack on this tick.");
                let opp_positions = self.get_opponent_positions(source_id, ctx.tick);
                let source = self.live_units.get_mut(&source_id).unwrap();
                if let Some(target) = source.find_target(&opp_positions) { // same attack but to other target
                    new_events.push((AttackEvent(AttackData{target, ..data}), ctx.tick))
                } else {
                    //println!("No more targets found!");
                }
                return (new_events, new_dead); // need to be careful here. 
            }

            let target = self.live_units.get_mut(&data.target).unwrap(); // safe unwrap bc of is_none() arm above.

            // check target is in range. If not, queue MoveEvent on tick.
            let target_distance = source_pos.distance_squared_to(&target.get_position(ctx.tick));
            if target_distance > source_range {

                // add MoveEvent. 
                let source = self.live_units.get(&data.source).unwrap();

                #[cfg(test)] 
                {
                    use crate::core_structs::unit::spatial_functions::LOGICAL_SUBTILES;
                    println!("tick {}: \t{} is out of range! distance: {}, range: {}", ctx.tick, _source_name, (target_distance.0 as f32).sqrt()/LOGICAL_SUBTILES as f32, source_range.0.isqrt()/LOGICAL_SUBTILES);
                }
                let m = MoveEvent(source.path(&self, ctx.tick));
                return (vec![(m,  ctx.tick)], vec![])
            }
            

            // roll for crit
            let did_crit = source_cc.did_crit(&mut self.rng);
            if did_crit {
                data.damage = Hitpoints(data.damage.0 * 2);
            };

            _pre_hp = target.current_hp;    

            if let Some(DeathEvent(id)) = target.take_damage(data) {
                new_dead.push(id);
            }

            _target_name = target.template.get_name();
            _max_hp = target.max_hp;
            _rem_hp = target.current_hp;
            _successful_hit = true;
   
            // handle source stuff e.g adding mana.
            let source = self.live_units.get_mut(&source_id).unwrap();
            let _source_name = source.template.get_name();
            if _successful_hit {
                source.mana.add(10, source.max_mana);
                //print!("tick {}: \t{} {}struck {} for {} damage!\t", ctx.tick.0, source_name, if did_crit {"critically "} else {""}, _target_name, _pre_hp.0 - _rem_hp.0);
                //print!("Mana {}/{}\t", source.mana.0, source.max_mana.0);
                //println!("Enemy HP {}/{}", _rem_hp.0, _max_hp.0);
            } 
        }
        (new_events, new_dead)
    }

    pub(super) fn process_ability_cast(&mut self, data: AbilityData, tick: u32) -> Vec<(BattleEvent, u32)> {
        // check if ability is targeted to current attack target. - if it is, ensure current target is valid.
        let source = self.live_units.get(&data.source).unwrap();
        if data.ability.target_paradigm == TargetParadigm::CurrentTarget && source.target.is_none() {
            let opp_pos = &self.get_opponent_positions(data.source, tick);
            let source = self.live_units.get_mut(&data.source).unwrap();
            source.find_target(opp_pos);
        }
        { // set mana to 0
            let source = self.live_units.get_mut(&data.source).unwrap();
            source.mana = Mana(0)
        }
        // apply ability effect to each target
        data.ability.cast(data.source, &self, tick)
    }

    pub(super) fn process_buff_event(&mut self, data: BuffData) {
        let target_unit = self.live_units.get_mut(&data.target).unwrap();
        let buff_bin = match data.buff.buff_type {
            BuffEffect::AttackDamageModifier(_)  => &mut target_unit.temp_stat_modifiers,
            BuffEffect::FlatIncomingReduction(_) => &mut target_unit.incoming_damage_handlers
        }; 
        let _ = *buff_bin.entry((data.buff.id, data.source))      // find entry if it exists
                        .and_modify(|container| container.add_stack(data.buff.buff_type.get_magnitude()))   // and change it by adding 1 stack with the current magnitude.                 
                        .or_insert(BuffContainer::new_from(data.buff)); // or create one with value 1.
    }

    pub(super) fn process_move_event(&mut self, data: MoveData) -> Vec<(BattleEvent, u32)> {
        let target_unit = self.live_units.get_mut(&data.target).unwrap(); // event would be flushed if target was dead
        target_unit.current_movement = Some(data);

        #[cfg(test)]
        println!("tick {}:  \t{} {} is moving to {} over {} ticks ", data.start_tick, target_unit.template.get_name(), target_unit.position, data.end_pos, data.end_tick-data.start_tick);

        self.blocked.set_coord(&data.end_pos, true);
        // queue movement end event
        vec![(MoveEndEvent(MoveEndData {
            target: data.target,
            end_pos: data.end_pos
        }), data.end_tick)]
    }

    pub(super) fn process_move_end_event(&mut self, data: MoveEndData, tick: u32) -> Option<BattleEvent> {
        let mut move_again = false; // if new move event should be instantly triggered
        {   
            let target_unit = self.live_units.get_mut(&data.target).unwrap(); // event would be flushed if target was daed

            // free the location they departed from
            self.blocked.set_coord(&target_unit.position, false);

            // update unit position variables
            target_unit.current_movement = None;
            target_unit.position = data.end_pos;
            
            #[cfg(test)]
            {
                println!("tick {}:  \t{} has arrived at {}", tick, target_unit.template.get_name(), data.end_pos);
            }
        }
        // double grab so needs immutable 
        // check if in range to requeue move event if necessary.
        let attacker = self.live_units.get(&data.target).unwrap();
        if let Some(unit) = attacker.target {
            if let Some(eid) = &self.live_units.get(&unit) {
                let target_distance = attacker.position.to_logical().distance_squared_to(&eid.get_position(tick));
                if target_distance > attacker.range_squared {
                    move_again = true
                }
            } 
        }

        // slipping in some debugging stuff here.
        #[cfg(test)] 
        {
            println!("{}", self.blocked)
        }
        // end debugging

        if move_again {
            Some(MoveEvent(attacker.path(&self, tick)))
        } else {None}
    }

    pub(super) fn process_debug_event(&mut self, data: DebugData) {
        //println!();
        match data.info {
            DebugInfo::_Health => {
                for unit in self.live_units.iter() {
                    let name = unit.template.get_name();
                    println!("{} {} - {}/{}hp", unit.id.0, name, unit.current_hp.0, unit.max_hp.0)
                }
            }
            DebugInfo::_Positions => {
                for unit in self.live_units.iter() {
                    let name = unit.template.get_name();
                    println!("{} {} - ({}, {})", unit.id.0, name, unit.position.x, unit.position.y)
                }
            }
            DebugInfo::_HealthAndPos => {
                for unit in self.live_units.iter() {
                    let name = unit.template.get_name();
                    println!("{} {} at ({}, {}) - {}/{}hp", unit.id.0, name, unit.position.x, unit.position.y, unit.current_hp.0, unit.max_hp.0)
                }
            }
            _ => unimplemented!()
        }
    }
}