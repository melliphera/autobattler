use std::collections::HashMap;

use crate::prelude::*;

#[derive(Clone)] // Clone is cheap because all non-collection primitives are Copy
pub(crate) struct BattleUnit {
    // Represents a single entity within a battle scenario. 
    pub(crate) id: EntityID,               // ID for entity tracking - *NOT* actual unit's id.
    pub(crate) unit: UnitTemplateID,
    pub(crate) team: Team,

    // positional data
    pub(crate) position: BattlePosition,  // NOT UPDATED WHILE MOVING
    pub(crate) move_speed: MoveSpeed,
    pub(crate) current_movement: Option<MoveData>,

    // defensive data
    pub(crate) current_hp: Hitpoints,
    pub(crate) max_hp: Hitpoints,
    pub(crate) defence: Mitigation,
    pub(crate) magic_resist: Mitigation,        
    
    // offensive data
    pub(crate) target: Option<EntityID>,
    pub(crate) attack_type: DamageType,      
    pub(crate) attack: AttackDamage,         
    pub(crate) attack_delay: AttackTickDelay,
    pub(crate) range_squared: AttackRange, // squared for distance comparisons as absolute distance is not needed.
    pub(crate) crit_chance: CritChance,

    // ability data
    pub(crate) ability: Option<Ability>,
    pub(crate) mana: Mana,
    pub(crate) max_mana: Mana,
    
    // buff data
    pub(crate) shield: Shield,
    pub(crate) incoming_damage_handlers: HashMap<(BuffID, EntityID), BuffContainer>, // includes both buffs and debuffs. - u8 = stacks.
    pub(crate) outgoing_damage_handlers: HashMap<(BuffID, EntityID), BuffContainer>, // includes both buffs and debuffs.
    pub(crate) temp_stat_modifiers:      HashMap<(BuffID, EntityID), BuffContainer>  // includes both buffs and debuffs.
}

impl BattleUnit {
    pub(crate) fn find_target(&mut self, enemy_positions: &Vec<(EntityID, BattlePosition)>) -> Option<EntityID> {
        // simple nearest-targeting logic for now.
        let mut closest_target: Option<EntityID> = None;
        let mut closest_distance: Option<AttackRange> = None;

        for (id, pos) in enemy_positions.iter() {
            let dist = self.position.distance_squared_to(pos);
            match closest_distance {
                None => {
                    closest_distance = Some(dist);
                    closest_target = Some(*id);
                }
                Some(current_closest) => {
                    if dist.0 < current_closest.0 {
                        closest_distance = Some(dist);
                        closest_target = Some(*id);
                    }
                }
            }
        }
        self.target = closest_target;
        closest_target
    }

    pub(crate) fn attack_current_target(&self) -> BattleEvent {
        // Attack target. This function is called to queue an attack rather than execute it so shouldn't modify anything about the player state.
        let mut damage = Hitpoints(self.attack.0);
        for (_buff, container) in self.outgoing_damage_handlers.iter() {
            damage = container.to_single().modify_damage(damage)
        }
        AttackEvent(
            AttackData { 
                source: self.id, 
                target: self.target.unwrap(), // safe unwrap because we've already established target is not None
                damage_type: self.attack_type,
                damage: Hitpoints(damage.0)
            }  
        )
    }

    pub(crate) fn take_damage(&mut self, incoming: AttackData) -> Option<BattleEvent> {
        let mut damage = Hitpoints(incoming.damage.0);
        // pass damage through buffs/debuffs here.
        for (_buff, container) in self.incoming_damage_handlers.iter() {
            damage = container.to_single().modify_damage(damage)
        }

        damage = match incoming.damage_type {
            Physical => { damage - Hitpoints(self.defence.0 as u64) }
            Magic    => { damage - Hitpoints(self.magic_resist.0 as u64)}
        };

        match self.shield.0 {
            None => self.current_hp -= damage.min(self.current_hp),
            Some(shield) => {
                if shield >= damage {self.shield = Shield(Some(shield-damage))}
                else { self.shield = Shield(None) }
            }
        }
        if self.current_hp.0 == 0 {
            Some(DeathEvent(self.id))
        } else {None}
    }

    pub(crate) fn get_position(&self, tick: u32) -> BattlePosition {
        if self.current_movement.is_none() {
            self.position
        } else {
            let move_order = self.current_movement.unwrap(); // safe unwrap bc above.
            let tick_number = tick - move_order.start_tick;
            let progress = tick_number as f32 / (move_order.end_tick - move_order.start_tick) as f32;
            let (x_f, y_f) = (
                move_order.start_pos.x as f32 + progress*(move_order.end_pos.x - move_order.start_pos.x) as f32,
                move_order.start_pos.y as f32 + progress*(move_order.end_pos.y - move_order.start_pos.y) as f32,
            );
            BattlePosition{ x: x_f as i32, y: y_f as i32 }
        }
    }

    pub(crate) fn path(&self, b: &BattleState, current_tick: u32) -> MoveData {
        // pathfinding logic for moving towards target. Returns MoveData object which can be processed as a MoveEvent
        let target = b.live_units.get(&self.target.expect("Unit tried to path without target!")).unwrap(); // should never issue MoveEvent without an active target - how would it know its out of range?
        let next = self.position.best_next_tile(&target.get_position(current_tick), self.range_squared);
        let travel_ticks = self.position.distance_squared_to(&next).0.isqrt() / self.move_speed.0;
        MoveData {
            source: self.id, 
            target: self.id, 
            start_pos: self.position, // can use position field directly as it will never path while under a MoveEvent.
            end_pos: next,
            start_tick: current_tick,
            end_tick: current_tick + travel_ticks as u32
        }
    }
}