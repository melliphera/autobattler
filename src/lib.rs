//! Outward facing impls for BattleState, to communicate with Godot
use rand::Rng;

#[allow(dead_code)]
mod core_structs;
use core_structs::prelude::*;

#[cfg(test)]
mod dev_tools;


pub use core_structs::battle::battle_state::BattleState;

impl BattleState {
    pub fn get_units(&self) -> Vec<&BattleUnit> {
        self.live_units.iter().collect()
    }
}

pub const MAX_FIGHT_LENGTH: u32 = 30000;

//#[cfg(test)]
pub mod tests {
    use super::*;
    use Roster::*;

    #[test]
    fn it_works() {
        let human_team = [
            (0, (5, 3)), // knight
            (1, (1, 3)), // ranger
            (2, (1, 2))  // mage
        ];

        let npc_team = [
            (0, (8, 1)),
            (0, (8, 2)),
            (0, (8, 3)),
            (0, (8, 4)),
            (0, (8, 5)),
        ];

        let mut b = BattleState::new_seeded(9405163005650660990)
                        //.with_debug(prelude::DebugInfo::_EventQueue, 500)
                        //.with_debug(prelude::DebugInfo::_HealthAndPos, 500)
                        ;
        for (id, (x, y)) in human_team.iter() {
            b.spawn_ally_from_id(UnitTemplateID(*id, Human), BattlePosition { x: *x, y: *y });
        }

        for (id, (x, y)) in npc_team.iter() {
            b.spawn_enemy_from_id(UnitTemplateID(*id, NPC),  BattlePosition { x: *x, y: *y });
        }

        b.simulate(MAX_FIGHT_LENGTH);
    }
    
    //#[test]
    pub fn profile() {
        let mut win_counter = [0; 3];
        let start = std::time::Instant::now();

        for _i in 0..1_000_000 {
            let mut human_team: Vec<(u16, (i32, i32))> = Vec::new();
            let mut npc_team: Vec<(u16, (i32, i32))> = Vec::new();

            let mut rng = rand::rng();

            let human_count = rng.random_range(2..=4);
            for _ in 0..human_count {
                let id = rng.random_range(0..=2);
                let x = rng.random_range(1..=5);
                let y = rng.random_range(1..=5);
                human_team.push((id, (x, y)));
            }

            let enemy_count = rng.random_range(3..=5);
            for _ in 0..enemy_count {
                let id = rng.random_range(0..=1);
                let x = rng.random_range(6..=10);
                let y = rng.random_range(1..=5);
                npc_team.push((id, (x, y)));
            }

            let mut b = BattleState::new_seeded(9405163005650660990);
            for (id, (x, y)) in human_team.iter() {
                b.spawn_ally_from_id(UnitTemplateID(*id, Human), BattlePosition { x: *x, y: *y });
            }

            for (id, (x, y)) in npc_team.iter() {
                b.spawn_enemy_from_id(UnitTemplateID(*id, NPC),  BattlePosition { x: *x, y: *y });
            }

            match b.simulate(MAX_FIGHT_LENGTH) {
                Some(Team::Player)   => {win_counter[0] += 1}
                Some(Team::Opponent) => {win_counter[1] += 1}
                None                 => {win_counter[2] += 1}
            };
        }
        println!("Time taken: {}s\nAlly wins: {}\nEnemy wins: {}, Fights timed out: {}", start.elapsed().as_secs_f32(), win_counter[0], win_counter[1], win_counter[2])
    }
}
