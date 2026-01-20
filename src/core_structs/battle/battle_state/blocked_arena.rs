//! Contains information for BlockedArena and LocationTag

use crate::prelude::{BattleSubtile, GridPosition};

use std::fmt::Display;

const DIRECTION_VECTORS: [(i32, i32); 8] = [
    (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1), (0, 1)
];
const ARENA_WIDTH: i32 = 10;
const ARENA_HEIGHT: i32 = 5;
const EMPTY_ARENA: [[bool; ARENA_HEIGHT as usize]; ARENA_WIDTH as usize] = [[false; ARENA_HEIGHT as usize]; ARENA_WIDTH as usize];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockedArena {
    // array grid defining each tile on the 2D arena. if a slot is True, it is blocked - either occupied or the end location of an existing move.
    pub data: [[bool; 5]; 10]
}

impl BlockedArena {
    pub const fn new() -> Self {
        BlockedArena { data: EMPTY_ARENA }
    }

    pub fn get_coord(&self, coord: &GridPosition) -> bool {
        // ASSUMES BATTLEPOSITION HAS BEEN SHRUNK TO TILE ALREADY.
        // therefore any non (0, 0) check would instantly be an out-of-bounds panic.
        self.data[coord.x as usize][coord.y as usize]
    }

    pub fn get_coord_from_logical(&self, coord: &BattleSubtile) -> bool {
        // helper function for non-tile based positions.
        self.get_coord(&coord.to_grid())
    }

    pub fn set_coord(&mut self, coord: &GridPosition, value: bool) {
        self.data[coord.x as usize][coord.y as usize] = value
    }

    pub fn get_adjacent_states(&self, base: &GridPosition) -> [bool; 8] {
        // gets the state of the 8 adjacent tiles from base. true means blocked.
        // automatically does bounds checking and returns false for any tile that would be out of bounds.
        #[cfg(test)]
        println!("Getting adjacent occupancy for tile ({}, {}).", base.x, base.y);

        let mut out = [true; 8]; // assume blocked by default, so no edits need to be made on out-of-bounds.

        for (i, (x_diff, y_diff)) in DIRECTION_VECTORS.iter().enumerate() {
            let (hypo_x, hypo_y) = (base.x + x_diff, base.y + y_diff);
            if hypo_x >= 0 && hypo_x < ARENA_WIDTH && hypo_y >= 0 && hypo_y < ARENA_HEIGHT {
                out[i] = self.data[hypo_x as usize][hypo_y as usize]
            }
        }
        out
    }
}

impl Display for BlockedArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let positions = self.data;
        let mut buf = String::with_capacity(130);

        buf += "\n  0 1 2 3 4\n";
        for (i, row) in positions.iter().enumerate() {
            buf += &format!("{} ", i);
            for bool_filled in row.into_iter() {
                buf += &format!("{} ", if *bool_filled {"O"} else {" "});
            }
            buf += "\n"
        }
        write!(f, "{}", buf)
    }
    

}