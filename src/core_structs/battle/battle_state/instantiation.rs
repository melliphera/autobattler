use rustc_hash::FxHasher;
use std::{collections::HashMap};
use std::cell::RefCell;
use std::hash::BuildHasherDefault;

use priority_queue::PriorityQueue;
use rand::{SeedableRng, rngs::StdRng};

use crate::core_structs::{battle::battle_state::entity_list::EntityList, prelude::*};

impl BattleState {
    pub fn _new() -> Self {
        let seed: u64 = rand::random();
        //println!("Generating new BattleState with seed: {}", seed);
        BattleState::new_seeded(seed)
    }

    pub fn new_seeded(seed: u64) -> Self {
        BattleState { 
            live_units: EntityList::new(), 
            timeline: PriorityQueue::with_hasher(BuildHasherDefault::<FxHasher>::default()), 
            rng: StdRng::seed_from_u64(seed),
            events_called: 0
        }
    }

    pub fn spawn_ally_from_id(&mut self, id: UnitTemplateID, position: BattlePosition) -> Result<(), ()> {
        let template = UNIT_DATABASE[id.0 as usize];
        let ability = get_ability(id);
        let b = BattleUnit { 
            id: self.live_units.get_next_id(), 
            template: id, 
            team: Team::Player,
            position: position.to_logical(),
            last_position: position.to_logical(),
            current_movement: None,
            blocked_on_last_move: RefCell::new(None),
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
            move_speed: template.move_speed.to_logical(),
            defence: template.defence, 
            magic_resist: template.magic_resist, 
            attack_type: template.attack_type, 
            attack: template.attack, 
            attack_delay: template.attack_delay, 
            crit_chance: template.crit_chance, 

        };
        self.live_units.spawn(b)?;
        Ok(())
    }

    pub fn spawn_enemy_from_id(&mut self, id: UnitTemplateID, position: BattlePosition) -> Result<(), ()> {
        let template = match id.1 {
            Roster::Human => UNIT_DATABASE[id.0 as usize],
            Roster::NPC  => ENEMY_DATABASE[id.0 as usize]
        };
        let ability = get_ability(id);

        let b = BattleUnit {
            id: self.live_units.get_next_id(),
            template: id, 
            team: Team::Opponent,
            position: position.to_logical(),
            last_position: position.to_logical(),
            current_movement: None,
            move_speed: template.move_speed.to_logical(),
            blocked_on_last_move: RefCell::new(None),
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
        self.live_units.spawn(b)?;
        Ok(())
    }

    pub fn initialize(&mut self) {
        // load any start-of-fight state (not implemented)
        //println!("Initialising battlefield.");
        let ally_positions = self.get_positions_by_team(Team::Player, 0);
        let opponent_positions = self.get_positions_by_team(Team::Opponent, 0);
        let mut initial_event_stack: Vec<BattleEvent> = Vec::with_capacity(self.live_units.len());

        for unit in self.live_units.iter_mut() {
            let target_positions = if unit.team == Team::Player {opponent_positions.clone()} else {ally_positions.clone()};
            unit.find_target(&target_positions);
            let battle_event = unit.attack_current_target();
            initial_event_stack.push(battle_event);
            
        }
        for event in initial_event_stack.iter() {
            self.queue_event(*event, 0);
        }
    }

    pub(crate) fn _with_debug(mut self, info: DebugInfo, interval: u32) -> Self {
        let payload = _DebugEvent(DebugData { delay: interval, info });
        self.queue_event(payload, 0);
        self
    }
}