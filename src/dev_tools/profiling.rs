use crate::core_structs::prelude::*;
use Roster::*;

use crate::MAX_FIGHT_LENGTH;
use rand::Rng;

use std::fmt::Display;
use std::thread;
use std::sync::mpsc;
use std::time::Instant;

use std::ops::AddAssign;

const TESTS_PER_POLL: usize = 100;

pub fn profile(seconds: usize) {
    let mut cumulative_results = ChunkResult::new();
    let start = Instant::now();

    let mut total_sims = 0;

    for p in 1..=seconds {
        while start.elapsed().as_secs_f32() < p as f32 {
            cumulative_results += profile_chunk(TESTS_PER_POLL);
            total_sims += TESTS_PER_POLL;
        }
        println!("{} simulations completed in {:.2}s", total_sims, start.elapsed().as_secs_f32())
    }
    println!("Time taken: {}s\n{}Events/s: {}", start.elapsed().as_secs_f32(), cumulative_results, cumulative_results.event_counter as u64/start.elapsed().as_secs())
}

pub fn profile_threaded(seconds: u64, threads: usize) {
    //! multithreaded implementation of profile(), running many sims per second.
    //! main thread runs no sims and is only used for communication
    let (tx, rx) = mpsc::channel::<ChunkResult>();
    let start = Instant::now();

    let mut cumulative_result = ChunkResult::new();
    let mut sims_run = 0;

    for i in 0..threads {
        let t = tx.clone();
        let timer_local = start.clone();
        thread::spawn(move || {
            println!("Thread{} started at {}μs", i, timer_local.elapsed().as_micros());
            let mut last_crossed_second = 0;

            let mut second_result = ChunkResult::new();

            while last_crossed_second < seconds {
                while timer_local.elapsed().as_secs() < last_crossed_second + 1 {
                    second_result += profile_chunk(TESTS_PER_POLL);
                }
                // send and reset counter
                t.send(second_result).unwrap();
                last_crossed_second += 1;
                second_result = ChunkResult::new()
            }
            println!("Thread {} has finished working.", i)
        });
    }

    // receiver thread
    drop(tx);
    let mut last_crossed_second = 0;

    for received in rx {
        cumulative_result += received;

        if start.elapsed().as_secs() == last_crossed_second + 1 {
            println!("{} simulations completed in {:.2}s", cumulative_result.sims_run, start.elapsed().as_secs_f32());
            last_crossed_second += 1
        }
    }
    println!("Time taken: {}s\n{}Events/s: {}", start.elapsed().as_secs_f32(), cumulative_result, cumulative_result.event_counter as u64/start.elapsed().as_secs())
}

pub struct ChunkResult {
    pub win_counter: [i32; 3],
    pub event_counter: i64,
    pub sims_run: usize
}

impl ChunkResult { 
    fn new()                 -> Self {ChunkResult { win_counter: [0; 3], event_counter: 0, sims_run: 0 }}
    fn with_tests(n: usize)  -> Self {ChunkResult { win_counter: [0; 3], event_counter: 0, sims_run: n }}
}

impl AddAssign for ChunkResult {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..3 {
            self.win_counter[i] += rhs.win_counter[i]
        }
        self.event_counter += rhs.event_counter;
        self.sims_run += rhs.sims_run;
    }
}

impl Display for ChunkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ally wins: {}\nEnemy wins: {}\nFights timed out: {}\n\nEvents called: {}\n", self.win_counter[0], self.win_counter[1], self.win_counter[2], self.event_counter)
    }
}


fn profile_chunk(num_tests: usize) -> ChunkResult {
    //! PASSES BACK ([ALLY WINS, ENEMY WINS, TIMEOUTS], EVENTS HANDLED)
    let mut out = ChunkResult::with_tests(num_tests);

    for _i in 0..num_tests {
        let result = random_test();
        out.event_counter += result.0 as i64;

        match result.1 {
            Some(Team::Player)   => {out.win_counter[0] += 1}
            Some(Team::Opponent) => {out.win_counter[1] += 1}
            None                 => {out.win_counter[2] += 1}
        };
    }
    out
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