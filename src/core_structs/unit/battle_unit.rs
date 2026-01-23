use std::collections::HashMap;
use std::cell::RefCell;

use fixedstr::str32;
use smallvec::SmallVec;

use crate::core_structs::battle::battle_state::blocked_arena::BlockedArena;
use crate::core_structs::battle::battle_state::team_position_caches::LocationTag;
use crate::core_structs::prelude::*;

#[derive(Clone, Debug)] // Clone is cheap because all non-collection primitives are Copy
pub struct BattleUnit {
    // Represents a single entity within a battle scenario. 
    pub id: EntityID,               // ID for entity tracking - *NOT* actual unit's id.
    pub template: UnitTemplateID,
    pub team: Team,

    // positional data
    pub position: GridPosition,  // NOT UPDATED WHILE MOVING
    pub last_position: GridPosition,
    pub move_speed: MoveSpeed,
    pub current_movement: Option<MoveData>,
    pub blocked_on_last_move: RefCell<Option<BlockedArena>>,

    // defensive data
    pub current_hp: Hitpoints,
    pub max_hp: Hitpoints,
    pub defence: Mitigation,
    pub magic_resist: Mitigation,        
    
    // offensive data
    pub target: Option<EntityID>,
    pub attack_type: DamageType,      
    pub attack: AttackDamage,         
    pub attack_delay: AttackTickDelay,
    pub range_squared: SquaredLogicalRange, // squared for distance comparisons as absolute distance is not needed.
    pub crit_chance: CritChance,

    // ability data
    pub ability: Option<Ability>,
    pub mana: Mana,
    pub max_mana: Mana,
    
    // buff data
    pub shield: Shield,
    pub incoming_damage_handlers: HashMap<(BuffID, EntityID), BuffContainer>, // includes both buffs and debuffs. - u8 = stacks.
    pub outgoing_damage_handlers: HashMap<(BuffID, EntityID), BuffContainer>, // includes both buffs and debuffs.
    pub temp_stat_modifiers:      HashMap<(BuffID, EntityID), BuffContainer>, // includes both buffs and debuffs.
}

impl BattleUnit {
    pub(crate) fn find_target(&mut self, enemy_positions: &SmallVec<[LocationTag; MAX_TEAM_SIZE]>) -> Option<EntityID> {
        // simple nearest-targeting logic for now.
        let mut closest_target: Option<EntityID> = None;
        let mut closest_distance: Option<SquaredLogicalRange> = None;

        for tag in enemy_positions.iter() {
            let dist = self.position.to_logical().distance_squared_to(&tag.location);
            match closest_distance {
                None => {
                    closest_distance = Some(dist);
                    closest_target = Some(tag.id);
                }
                Some(current_closest) => {
                    if dist.0 < current_closest.0 {
                        closest_distance = Some(dist);
                        closest_target = Some(tag.id);
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
                damage: Hitpoints(damage.0),
                caused_by: "attack".into()
            }  
        )
    }

    pub(crate) fn take_damage(&mut self, incoming: AttackData) -> (Hitpoints, Option<BattleEvent>) {
        //! returns the actual damage taken (for hitsplats/event logs) and a DeathEvent if the target dies.
        
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
        if incoming.caused_by != str32::from("attack") {
            //println!("{} took {} damage from {}!", self.template.get_name(), damage.0, incoming.caused_by)
        }

        if self.current_hp.0 == 0 {
            (damage, Some(DeathEvent(self.id)))
        } else { (damage, None) }
    }

    pub(crate) fn heal(&mut self, incoming: HealData) -> Hitpoints {
        // returns the amount healed for combat logging/hitsplat purposes.
        let start_hp = self.current_hp;
        if incoming.can_overheal {
            self.current_hp += incoming.amount
        } else {
            self.current_hp = Hitpoints((self.current_hp.0 + incoming.amount.0).min(self.max_hp.0))
        }
        self.current_hp - start_hp
    }

    pub(crate) fn shield(&mut self, incoming: ShieldData) {
        // shielding doesn't stack. 10 shield + 50 shield = 50 shield.
        if let Some(hp) = self.shield.0 { if hp > incoming.amount {
            return // current shield is higher so do nothing
        }}
        self.shield = Shield(Some(incoming.amount))
    }

    pub(crate) fn get_position(&self, tick: Tick) -> (BattleSubtile, bool) {
        //! BattleSubtile is position, bool is whether it is actively moving or not (for cacheing reasons).

        if self.current_movement.is_none() {
            (self.position.to_logical(), false)
        } else {
            #[cfg(test)] {
                //println!("Getting position of moving unit with MoveData:- current tick: {}\n{:?} ", tick.0, self.current_movement.unwrap())
            }
            let move_order = self.current_movement.unwrap(); // safe unwrap bc above.
            let tick_number = tick.0 - move_order.start_tick.0;
            let progress = tick_number as f32 / (move_order.end_tick.0 - move_order.start_tick.0) as f32;
            let (x_f, y_f) = (
                move_order.start_pos.x as f32 + progress*(move_order.end_pos.x - move_order.start_pos.x) as f32,
                move_order.start_pos.y as f32 + progress*(move_order.end_pos.y - move_order.start_pos.y) as f32,
            );
            (BattleSubtile{ x: x_f as i32, y: y_f as i32 }, true)
        }
    }

    pub(crate) fn path(&self, blocked: &BlockedArena, target_pos: BattleSubtile, current_tick: Tick) -> MoveData {
        let mut blocked = *blocked; // create editable local copy

        let tile_coords = self.last_position;
        blocked.set_coord(&tile_coords, true);

        //#[cfg(test)]
        //if self.position != self.last_position {
        //    assert_ne!(b.blocked, blocked, "blocked is copying to b.blocked!");
        //}

        // check against cache for situations with fully blocked movement.
        {
            let b = self.blocked_on_last_move.borrow();
            if !b.is_none() && b.as_ref().unwrap() == &blocked {
                // return another 10 tick "move" to current location.
                return 
                    MoveData {
                        source: self.id, 
                        target: self.id, 
                        start_pos: self.position, // can use position field directly as it will never path while under a MoveEvent.
                        end_pos: self.position,
                        start_tick: current_tick,
                        end_tick: current_tick + Tick(10),
                        move_speed_override: None
                }
            }
        }
        
        let next = self.position.best_next_tile(&target_pos, self.range_squared, &blocked);

        // if next is self, forcibly add 10 tick delay to not spam moveevents, and cache current blockedtiles.
        let travel_ticks = if next == self.position { 
            self.blocked_on_last_move.borrow_mut().replace(blocked);
            10 
        } else {
            // todo this line is disgustingly expensive for what it is.
            self.position.to_logical().distance_squared_to(&next.to_logical()).0.isqrt() / self.move_speed.0
        };
        MoveData {
            source: self.id, 
            target: self.id, 
            start_pos: self.position, // can use position field directly as it will never path while already under a MoveEvent.
            end_pos: next,
            start_tick: current_tick,
            end_tick: current_tick + Tick(travel_ticks as u16),
            move_speed_override: None
        }
    }
}