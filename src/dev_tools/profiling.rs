use crate::core_structs::battle::battle_state::godot_interface::godot_events::GodotEvent;
use crate::core_structs::prelude::*;
use Roster::*;

use crate::MAX_FIGHT_LENGTH;
use rand::Rng;

use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::sync::mpsc;
use std::time::Instant;

use std::ops::AddAssign;

const TESTS_PER_POLL: usize = 100;
const TEST_SEED: [u8; 32] = unsafe { std::mem::transmute::<[u128; 2], [u8; 32]>([9405163043505650660990, 376425672376467324256]) };
const CHARSET_16: [char; 16] = ['0','1','2','3','4','5','6','7','8','9','A','B','C','D','E','F'];


pub fn profile(seconds: usize) {
    let mut cumulative_results = ChunkResult::new();
    let start = Instant::now();

    let mut total_sims = 0;

    for p in 1..=seconds {
        while start.elapsed().as_secs_f32() < p as f32 {
            cumulative_results += profile_chunk(TESTS_PER_POLL, false);
            total_sims += TESTS_PER_POLL;
        }
        println!("{} simulations completed in {:.2}s", total_sims, start.elapsed().as_secs_f32())
    }
    println!("Time taken: {}s\n{}Events/s: {}", start.elapsed().as_secs_f32(), cumulative_results, cumulative_results.event_counter as u64/start.elapsed().as_secs())
}

pub fn profile_with_event_log(seconds: usize) {
    let mut cumulative_results = ChunkResult::new();
    let start = Instant::now();

    let mut total_sims = 0;

    for p in 1..=seconds {
        while start.elapsed().as_secs_f32() < p as f32 {
            cumulative_results += profile_chunk(TESTS_PER_POLL, true);
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

    for _i in 0..threads {
        let t = tx.clone();
        let timer_local = start.clone();
        thread::spawn(move || {
            #[cfg(test)]
            println!("Thread{} started at {}μs", _i, timer_local.elapsed().as_micros());

            let mut last_crossed_second = 0;
            let mut second_result = ChunkResult::new();

            while last_crossed_second < seconds {
                while timer_local.elapsed().as_secs() < last_crossed_second + 1 {
                    second_result += profile_chunk(TESTS_PER_POLL, false);
                }
                // send and reset counter
                t.send(second_result).unwrap();
                last_crossed_second += 1;
                second_result = ChunkResult::new()
            }

            #[cfg(test)]
            println!("Thread {} has finished working.", _i)
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

fn profile_chunk(num_tests: usize, with_listener: bool) -> ChunkResult {
    //! PASSES BACK ([ALLY WINS, ENEMY WINS, TIMEOUTS], EVENTS HANDLED)
    let mut out = ChunkResult::with_tests(num_tests);

    for _i in 0..num_tests {
        let result = if with_listener {
            random_with_listener()
        } else { 
            random_test()
        };
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

    let mut b = BattleState::new_seeded(TEST_SEED);
    for (id, (x, y)) in human_team.iter() {
        _ = b.spawn_ally_from_id(UnitTemplateID(*id, Human), GridPosition { x: *x, y: *y });
    }

    for (id, (x, y)) in npc_team.iter() {
        _ = b.spawn_enemy_from_id(UnitTemplateID(*id, NPC),  GridPosition { x: *x, y: *y });
    }

    b.simulate(MAX_FIGHT_LENGTH)
}

fn random_with_listener() -> (i32, Option<Team>) {
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

    let mut b = BattleState::new().with_godot_event_buffer();
    for (id, (x, y)) in human_team.iter() {
        _ = b.spawn_ally_from_id(UnitTemplateID(*id, Human), GridPosition { x: *x, y: *y });
    }

    for (id, (x, y)) in npc_team.iter() {
        _ = b.spawn_enemy_from_id(UnitTemplateID(*id, NPC),  GridPosition { x: *x, y: *y });
    }

    let r = b.simulate(MAX_FIGHT_LENGTH);
    match r.1 {
        None => {
            if let Some(ref buffer) = b.godot_event_buffer {
                let log_file = format!("./logs/{}.log", get_logstring(b.rng.get_seed()));
                let mut file = File::create(&log_file).expect(&format!("Failed to create log file at {}", log_file));
                file.write_all(format!("{}\nSTART OF EVENT TRANSCRIPT\n", parse_unit_manifest(b.unit_manifest.unwrap())).as_bytes()).expect("Failed to write to created logfile.");
                file.write_all(format!("{}", parse_event_buffer(buffer)).as_bytes()).expect("Failed to write to created logfile.");
            }
        }
        _ => {}
    }
    r
}

fn get_logstring(bytes: [u8; 32]) -> String {
    let mut out = String::with_capacity(32);
    for byte in bytes[0..8].into_iter() {
        out.push(CHARSET_16[(byte/16) as usize]);
        out.push(CHARSET_16[(byte%16) as usize]);
    }
    out
}

fn parse_event_buffer(buff: &Vec<GodotEvent>) -> String {
    let mut b = String::with_capacity(buff.len()*20);
    for event in buff.iter() {
        b += &(event.to_string() + "\n");
    }
    b
}

fn parse_unit_manifest(mani: Vec<(EntityID, fixedstr::str32)>) -> String {
    let mut b = String::with_capacity(mani.len()*20);
    b += "[\n";
    for unit in mani.iter() {
        b += &format!("\t{:?}\n", unit);
    }
    b += "]\n";
    b
}