use std::{collections::HashMap};
use std::cell::{RefCell};

use rand::{SeedableRng, rngs::StdRng};

use crate::core_structs::battle::battle_state::blocked_arena::BlockedArena;
use crate::core_structs::battle::battle_state::timeline::EventTimeline;
use crate::core_structs::{battle::battle_state::entity_list::EntityList, prelude::*};
use crate::Roster::*;

impl BattleState {
    pub fn _new() -> Self {
        let seed: u64 = rand::random();
        //println!("Generating new BattleState with seed: {}", seed);
        BattleState::new_seeded(seed)
    }

    pub fn new_seeded(seed: u64) -> Self {
        BattleState { 
            live_units: EntityList::new(), 
            timeline: EventTimeline::new(), 
            rng: StdRng::seed_from_u64(seed),
            events_called: 0,
            last_processed_tick: 0,
            blocked: BlockedArena::new(),
            godot_event_buffer: None
        }
    }

    pub fn new_with_teams(human_team: &[(u16, (i32, i32))], enemy_team: &[(u16, (i32, i32))]) -> BattleState {
        // mostly for quickly building combat scenarios for testing. Assumes enemy is NPC.
        let mut b = BattleState::_new();
        for (id, (x, y)) in human_team.iter() {
            _ = b.spawn_ally_from_id(UnitTemplateID(*id, Human), GridPosition  { x: *x, y: *y });
        };

        for (id, (x, y)) in enemy_team.iter() {
            _ = b.spawn_enemy_from_id(UnitTemplateID(*id, NPC),  GridPosition { x: *x, y: *y });
        };
        b
    }

    pub fn new_seeded_with_teams(seed: u64, human_team: &[(u16, (i32, i32))], enemy_team: &[(u16, (i32, i32))]) -> BattleState {
        // mostly for quickly building combat scenarios for testing. Assumes enemy is NPC.
        let mut b = BattleState::new_seeded(seed);
        for (id, (x, y)) in human_team.iter() {
            _ = b.spawn_ally_from_id(UnitTemplateID(*id, Human), GridPosition  { x: *x, y: *y });
        };

        for (id, (x, y)) in enemy_team.iter() {
            _ = b.spawn_enemy_from_id(UnitTemplateID(*id, NPC),  GridPosition { x: *x, y: *y });
        };
        b
    }

    fn spawn_from_id(&mut self, id: UnitTemplateID, position: GridPosition, team: Team) -> Result<(), ()> {
        let template = UNIT_DATABASE[id.0 as usize];
        let ability = get_ability(id);
        let b = BattleUnit { 
            id: self.live_units.get_next_id(), 
            template: id, 
            team: team,
            position: position,
            last_position: position,
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
        self.blocked.set_coord(&GridPosition { x: position.x, y: position.y}, true);
        self.live_units.spawn(b)?;
        Ok(())
    }

    pub fn spawn_ally_from_id(&mut self, id: UnitTemplateID, position: GridPosition) -> Result<(), ()> {
        self.spawn_from_id(id, position, Team::Player)
    }

    pub fn spawn_enemy_from_id(&mut self, id: UnitTemplateID, position: GridPosition) -> Result<(), ()> {
        self.spawn_from_id(id, position, Team::Opponent)
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

        #[cfg(test)]
        println!("{}", self.blocked)
    }

    pub(crate) fn _with_debug(mut self, info: DebugInfo, interval: u32) -> Self {
        let payload = _DebugEvent(DebugData { delay: interval, info });
        self.queue_event(payload, 0);
        self
    }
}