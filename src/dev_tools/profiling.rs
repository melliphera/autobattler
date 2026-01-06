use crate::core_structs::prelude::*;
use Roster::*;

use crate::MAX_FIGHT_LENGTH;
use rand::Rng;

const TESTS_PER_POLL: usize = 100;

pub fn profile(seconds: usize) {
    let mut win_counter = [0; 3];
    let start = std::time::Instant::now();

    let mut total_events: i64 = 0;
    let mut total_sims = 0;

    for p in 1..=seconds {
        while start.elapsed().as_secs_f32() < p as f32 {
            for _i in 0..TESTS_PER_POLL {
                let result = random_test();
                total_events += result.0 as i64;

                match result.1 {
                    Some(Team::Player)   => {win_counter[0] += 1}
                    Some(Team::Opponent) => {win_counter[1] += 1}
                    None                 => {win_counter[2] += 1}
                };
            }
            total_sims += TESTS_PER_POLL
        }
        println!("{} simulations completed in {:.2}s", total_sims, start.elapsed().as_secs_f32())
    }
    println!("Time taken: {}s\nAlly wins: {}\nEnemy wins: {}\nFights timed out: {}\nEvents called: {}", start.elapsed().as_secs_f32(), win_counter[0], win_counter[1], win_counter[2], total_events)
}

fn random_test() -> (i32, Option<Team>) {
    let mut human_team: Vec<(u16, (i32, i32))> = Vec::new();
    let mut npc_team: Vec<(u16, (i32, i32))> = Vec::new();

    let mut rng = rand::rng();

    let human_count = rng.random_range(2..=4);
    for _ in 0..human_count {
        let id = rng.random_range(0..=2);
        let x  = rng.random_range(0..=4);
        let y  = rng.random_range(0..=4);
        human_team.push((id, (x, y)));
    }

    let enemy_count = rng.random_range(3..=5);
    for _ in 0..enemy_count {
        let id = rng.random_range(0..=1);
        let x  = rng.random_range(5..=9);
        let y  = rng.random_range(0..=4);
        npc_team.push((id, (x, y)));
    }

    let mut b = BattleState::new_seeded(9405163005650660990);
    for (id, (x, y)) in human_team.iter() {
        _ = b.spawn_ally_from_id(UnitTemplateID(*id, Human), GridPosition { x: *x, y: *y });
    }

    for (id, (x, y)) in npc_team.iter() {
        _ = b.spawn_enemy_from_id(UnitTemplateID(*id, NPC),  GridPosition { x: *x, y: *y });
    }

    b.simulate(MAX_FIGHT_LENGTH)
}