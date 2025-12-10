use crate::core_structs::{
    unit::prelude::*,
    battle::battle_event::{*, BattleEvent::*},
};

#[derive(Clone)] // Clone is cheap because all non-collection primitives are Copy
pub struct BattleUnit {
    // Represents a single entity within a battle scenario. 
    pub id: EntityID,               // ID for entity tracking - *NOT* actual unit's id.
    pub unit: UnitTemplateID,
    pub team: Team,
    pub position: BattlePosition,        
    
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
    pub range_squared: AttackRange, // squared for distance comparisons as absolute distance is not needed.
    pub crit_chance: CritChance,
    
    // buff data
    pub shield: Shield,
    pub incoming_damage_handlers: Vec<Buff>, // includes both buffs and debuffs.
    pub outgoing_damage_handlers: Vec<Buff>, // includes both buffs and debuffs.
    pub temp_stat_modifiers: Vec<Buff> // includes both buffs and debuffs.
}

impl BattleUnit {
    pub fn find_target(&mut self, enemy_positions: &Vec<(EntityID, BattlePosition)>) -> Option<EntityID> {
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

    pub fn attack_current_target(&self) -> BattleEvent {
        // Attack target. This function is called to queue an attack rather than execute it so shouldn't modify anything about the player state.
        let mut damage = Hitpoints(self.attack.0);
        for buff in self.outgoing_damage_handlers.iter() {
            damage = buff.modify(damage)
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

    pub fn take_damage(&mut self, incoming: AttackData) -> Option<BattleEvent> {
        let mut damage = Hitpoints(incoming.damage.0);
        // pass damage through buffs/debuffs here.
        for buff in self.incoming_damage_handlers.iter_mut() {
            damage = buff.modify(damage)
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
}