//! Contains impl on BattleState for processing combat Events.
use fixedstr::str32;

use crate::core_structs::battle::battle_event::{AttackData, BattleEvent::{self, *}, MoveData};
use crate::core_structs::unit::prelude::*;
use crate::BattleState;
use std::cmp::Reverse;

pub(super) struct AttackContext {
    pub(super) event: BattleEvent,
    pub(super) tick: Reverse<u32>
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
        let source_pos: BattlePosition;
        let source_range: AttackRange;
        let source_ms: MoveSpeed;
        let source_name: str32;
        {
            let source_unit = self.live_units.get(&source_id).unwrap(); // safe because source is definitely still alive.
            source_cc = source_unit.crit_chance;
            source_pos = source_unit.get_position(ctx.tick.0);
            source_range = source_unit.range_squared;
            source_ms = source_unit.move_speed;
            source_name = source_unit.unit.get_name()
        }

        {   
            // handle target stuff - use different code block for source if necessary later.
            if self.live_units.get(&data.target).is_none() {
                println!("Target dead, requeueing new attack on this tick.");
                let opp_positions = self.get_opponent_positions(source_id, ctx.tick.0);
                let source = self.live_units.get_mut(&source_id).unwrap();
                if let Some(target) = source.find_target(&opp_positions) { // same attack but to other target
                    new_events.push((AttackEvent(AttackData{target, ..data}), ctx.tick.0))
                } else {
                    println!("No more targets found!");
                }
                return (new_events, new_dead);
            }

            let target = self.live_units.get_mut(&data.target).unwrap(); // safe unwrap bc of is_none() arm above.

            // check target is in range. If not, queue MoveEvent on tick.
            let target_distance = source_pos.distance_squared_to(&target.get_position(ctx.tick.0));
            if target_distance > source_range {

                // add MoveEvent. Nobody dies and no attacks are made, so can be returned directly.
                println!("tick {}: \t{} is out of range! distance: {}, range: {}", ctx.tick.0, source_name, (target_distance.0 as f32).sqrt()/4096f32, source_range.0.isqrt()/4096);
                let current_tick = ctx.tick.0;
                let next = source_pos.best_next_tile(&target.get_position(ctx.tick.0), source_range);
                let travel_ticks = source_pos.distance_squared_to(&next).0.isqrt() / source_ms.0;
                let m = MoveEvent( MoveData {
                    source: source_id, 
                    target: source_id, 
                    start_pos: source_pos,
                    end_pos: next,
                    start_tick: current_tick,
                    end_tick: current_tick + travel_ticks as u32
                });
                println!("{:#?}", m);
                return (vec![(m,  current_tick)], vec![])
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
            _target_name = target.unit.get_name();
            _max_hp = target.max_hp;
            _rem_hp = target.current_hp;
            _successful_hit = true;
   
            // handle source stuff e.g adding mana.
            let source = self.live_units.get_mut(&source_id).unwrap();
            let source_name = source.unit.get_name();
            if _successful_hit {
                source.mana.add(10, source.max_mana);
                print!("tick {}: \t{} {}struck {} for {} damage!\t", ctx.tick.0, source_name, if did_crit {"critically "} else {""}, _target_name, _pre_hp.0 - _rem_hp.0);
                print!("Mana {}/{}\t", source.mana.0, source.max_mana.0);
                println!("Enemy HP {}/{}", _rem_hp.0, _max_hp.0);
            } 
        }
        (new_events, new_dead)
    }
}