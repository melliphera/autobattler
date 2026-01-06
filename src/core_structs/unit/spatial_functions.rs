use crate::{core_structs::battle::battle_state::blocked_arena::BlockedArena, prelude::{GridPosition, SquaredLogicalRange}};

use super::primitives::{AttackRange, BattleSubtile};
use std::fmt::Display;

pub const LOGICAL_SUBTILES: i32 = 512;
pub const TICKS_PER_SECOND: i32 = 20;

impl AttackRange {
    pub(crate) fn to_squared(self) -> SquaredLogicalRange {
        SquaredLogicalRange((self.0*LOGICAL_SUBTILES).pow(2)) //  LOGICAL_SUBTILES "logical pixels" per tile
    }
}

impl GridPosition {
    pub(crate) fn to_logical(self) -> BattleSubtile {
        BattleSubtile { x: self.x * LOGICAL_SUBTILES, y: self.y * LOGICAL_SUBTILES }
    }

    pub(crate) fn best_next_tile(&self, other: &BattleSubtile, target_distance_squared: SquaredLogicalRange, blocked: &BlockedArena) -> GridPosition {
        // 0) walk directly to target if aligned and unblocked.
        let self_as_subtile = self.to_logical();
        if self_as_subtile.y == other.y {
            let direct = self.translated_by_tiles(((other.x - self_as_subtile.x).signum(), 0));
            if !blocked.get_coord(&direct) {
                return direct;
            }
        } else if self_as_subtile.x == other.x {
            let direct = self.translated_by_tiles((0, (other.y - self_as_subtile.y).signum()));
            if !blocked.get_coord(&direct) {
                return direct;
            }
        }

        // 1) check if orthogonal brings into range and is unblocked
        let (x_dist, y_dist) = (other.x - self_as_subtile.x, other.y - self_as_subtile.y);
        let best_orthogonal_direction: (i32, i32) = if x_dist.abs() == y_dist.abs() {
            (x_dist.signum(), 0)
        } else {
            let normaliser = x_dist.abs().max(y_dist.abs());
            (x_dist / normaliser, y_dist / normaliser)
        };
        let best_orthogonal_tile = self.translated_by_tiles(best_orthogonal_direction);
        if best_orthogonal_tile.to_logical().distance_squared_to(other) <= target_distance_squared && !blocked.get_coord(&best_orthogonal_tile) {
            return best_orthogonal_tile;
        }

        // 2) check if diagonal brings into range and is unblocked
        let diag_direction = (x_dist.signum(), y_dist.signum());
        let best_diagonal_tile = self.translated_by_tiles(diag_direction);

        if x_dist != 0 && y_dist != 0 && !blocked.get_coord(&best_diagonal_tile) { // seperating these checks out to pragmatically indicate that theyre far faster, skipping expensive code.
            if best_diagonal_tile.to_logical().distance_squared_to(other) <= target_distance_squared  {
                return best_diagonal_tile;
            }
        }


        // 3) use dot product to see if orthogonal or diagonal is better assuming both unblocked
        let dot_orthogonal = x_dist * best_orthogonal_direction.0 + y_dist * best_orthogonal_direction.1;
        let dot_diagonal = x_dist * diag_direction.0 + y_dist * diag_direction.1;

        if dot_diagonal.pow(2) > 2 * dot_orthogonal.pow(2) && !blocked.get_coord(&best_diagonal_tile) {
            return best_diagonal_tile;
        } else if !blocked.get_coord(&best_orthogonal_tile) {
            return best_orthogonal_tile;
        }

        // 4) look at all unblocked tiles that decrease either x_dist or y_dist and pick best by alignment.
        let direction_vectors = [
            (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1), (0, 1)
        ];

        let direction_is_blocked = blocked.get_adjacent_states(&self);

        let mut best_dir = (0, 0);
        let mut max_dot = i32::MIN;
        for ((x, y), blocked) in direction_vectors.iter().zip(direction_is_blocked) {
            if ((x == &x_dist.signum() && x != &0) || (y == &y_dist.signum()&& y != &0)) && ![diag_direction, best_orthogonal_direction].contains(&(*x, *y)) && !blocked {
                let dot = (x * x_dist + y * y_dist).pow(2) / (x_dist.abs() + y_dist.abs()); // div0 should never happen as that would require occupying the same tile.
                if dot > max_dot {
                    best_dir = (*x, *y);
                    max_dot = dot;
                }
            }
        }

        self.translated_by_tiles(best_dir)
    }

    pub(crate) fn translated_by_tiles(&self, tuple: (i32, i32)) -> GridPosition {
        GridPosition { x: self.x + tuple.0, y: self.y + tuple.1 }
    }

}

impl BattleSubtile {
    pub(crate) fn distance_squared_to(&self, other: &Self) -> SquaredLogicalRange {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        SquaredLogicalRange(dx*dx + dy*dy) // pythagorean
    }

    pub(crate) fn translated_by_tiles(&self, tuple: (i32, i32)) -> BattleSubtile {
        BattleSubtile { x: self.x + tuple.0*LOGICAL_SUBTILES, y: self.y + tuple.1*LOGICAL_SUBTILES }
    }

    pub(crate) fn to_grid(self) -> GridPosition {
        // as this is used for indexing, also needs to shift -1.
        GridPosition { x: self.x/LOGICAL_SUBTILES - 1, y: self.y/LOGICAL_SUBTILES - 1 }
    }
}

impl Display for BattleSubtile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LOGI({}, {})", self.x, self.y)
    }
}

impl Display for GridPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GRID({}, {})", self.x, self.y)
    }
}