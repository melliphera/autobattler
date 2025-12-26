use crate::prelude::SquaredLogicalRange;

use super::primitives::{AttackRange, BattlePosition};
use std::fmt::Display;

pub const LOGICAL_SUBTILES: i32 = 512;
pub const TICKS_PER_SECOND: i32 = 20;

impl AttackRange {
    pub(crate) fn to_squared(self) -> SquaredLogicalRange {
        SquaredLogicalRange((self.0*LOGICAL_SUBTILES).pow(2)) //  LOGICAL_SUBTILES "logical pixels" per tile
    }
}

impl BattlePosition {
    pub(crate) fn distance_squared_to(&self, other: &Self) -> SquaredLogicalRange {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        SquaredLogicalRange(dx*dx + dy*dy) // pythagorean
    }

    pub(crate) fn translated_by_tiles(&self, tuple: (i32, i32)) -> BattlePosition {
        BattlePosition { x: self.x + tuple.0*LOGICAL_SUBTILES, y: self.y + tuple.1*LOGICAL_SUBTILES }
    }


    pub(crate) fn to_logical(self) -> BattlePosition {
        BattlePosition { x: self.x * LOGICAL_SUBTILES, y: self.y * LOGICAL_SUBTILES }
    }

    pub(crate) fn best_next_tile(&self, other: &Self, target_distance_squared: SquaredLogicalRange, blocked: &Vec<BattlePosition>) -> BattlePosition {
        // 0) walk directly to target if aligned and unblocked.
        if self.y == other.y {
            let direct = self.translated_by_tiles(((other.x - self.x).signum(), 0));
            if !blocked.contains(&direct) {
                return direct;
            }
        } else if self.x == other.x {
            let direct = self.translated_by_tiles((0, (other.y - self.y).signum()));
            if !blocked.contains(&direct) {
                return direct;
            }
        }

        // 1) check if orthogonal brings into range and is unblocked
        let (x_dist, y_dist) = (other.x - self.x, other.y - self.y);
        let best_orthogonal_direction: (i32, i32) = if x_dist.abs() == y_dist.abs() {
            (x_dist.signum(), 0)
        } else {
            let normaliser = x_dist.abs().max(y_dist.abs());
            (x_dist / normaliser, y_dist / normaliser)
        };
        let best_orthogonal_tile = self.translated_by_tiles(best_orthogonal_direction);
        if best_orthogonal_tile.distance_squared_to(other) <= target_distance_squared && !blocked.contains(&best_orthogonal_tile) {
            return best_orthogonal_tile;
        }

        // 2) check if diagonal brings into range and is unblocked
        let diag_direction = (x_dist.signum(), y_dist.signum());
        let best_diagonal_tile = self.translated_by_tiles(diag_direction);
        if best_diagonal_tile.distance_squared_to(other) <= target_distance_squared && !blocked.contains(&best_diagonal_tile) {
            return best_diagonal_tile;
        }

        // 3) use dot product to see if orthogonal or diagonal is better assuming both unblocked
        let dot_orthogonal = x_dist * best_orthogonal_direction.0 + y_dist * best_orthogonal_direction.1;
        let dot_diagonal = x_dist * diag_direction.0 + y_dist * diag_direction.1;

        if dot_diagonal.pow(2) > 2 * dot_orthogonal.pow(2) && !blocked.contains(&best_diagonal_tile) {
            return best_diagonal_tile;
        } else if !blocked.contains(&best_orthogonal_tile) {
            return best_orthogonal_tile;
        }

        // 4) look at all unblocked tiles that decrease either x_dist or y_dist and pick best by alignment.
        let direction_vectors = [
            (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1), (0, 1)
        ];

        let mut best_dir = (0, 0);
        let mut max_dot = i32::MIN;
        for &(x, y) in &direction_vectors {
            if (x == x_dist.signum() || y == y_dist.signum()) && ![diag_direction, best_orthogonal_direction].contains(&(x, y)) {
                let dest = self.translated_by_tiles((x, y));
                if 0 <= dest.x && 0 <= dest.y && dest.x <= 9 * LOGICAL_SUBTILES && dest.y <= 4 * LOGICAL_SUBTILES && !blocked.contains(&dest) {
                    let dot = x * x_dist + y * y_dist;
                    if dot > max_dot {
                        best_dir = (x, y);
                        max_dot = dot;
                    }
                }
            }
        }

        self.translated_by_tiles(best_dir)
    }
}

impl Display for BattlePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x/LOGICAL_SUBTILES, self.y/LOGICAL_SUBTILES)
    }
}




    /* 
    pub(crate) fn best_next_tile_old(&self, other: &Self, target_distance: AttackRange, blocked: &Vec<BattlePosition>) -> BattlePosition {
        //! Chooses an adjacent (diagonal included) tile for a MoveEvent based on the following criteria:
        //! 0) if in same row or col as target, move closer along row or col.
        //! otherwise:
        //! 1) if best orthogonal tile puts within target_distance, do that
        //! 2) if best diagonal tile puts within target_distance, do that instead
        //! 3) if neither do, move in direction most closely aligned with target direction.
        //! 
        //! Diagonal is deliberately deprioritised via steps 2/3 because it takes 41% longer to walk there.
         
        // 0) walk directly to target if aligned and unblocked.
        if self.y == other.y {
            let direct = self.translated_by(((other.x - self.x).signum(), 0));
            if !blocked.contains(&direct) {
                return direct;
            }
        }
        else if self.x == other.x {
            let direct = self.translated_by((0, (other.y - self.y).signum()));
            if !blocked.contains(&direct) {
                return direct;
            }
        }

        // 1) check if orthogonal brings into range and is unblocked
        let (x_dist, y_dist) = (other.x - self.x, other.y - self.y);
        let best_orthogonal_direction: (i32, i32);

        // if target direction is perfectly diagonal, prioritise x movement
        if x_dist.abs() == y_dist.abs() {   
            best_orthogonal_direction = (x_dist.signum(), 0);

        } else {
            let normaliser = x_dist.abs().max(y_dist.abs());
            best_orthogonal_direction = (x_dist/normaliser, y_dist/normaliser); // because of floor division this returns either (+-1, 0) or (0, +-1) unless x=y as handled above
        }
        let best_orthogonal_tile = self.translated_by(best_orthogonal_direction);
        if best_orthogonal_tile.distance_squared_to(other) <= target_distance && !blocked.contains(&best_orthogonal_tile) {
            return best_orthogonal_tile;
        }

        // 2) check if diagonal brings into range and is unblocked
        let diag_direction = (x_dist.signum(), y_dist.signum());
        let best_diagonal_tile = self.translated_by(diag_direction);
        if best_diagonal_tile.distance_squared_to(other) <= target_distance && !blocked.contains(&best_diagonal_tile) {
            return best_diagonal_tile;
        }
        // 3) use squared dot product to see if orthogonal or diagonal is better assuming both unblocked
        let dot_orthogonal = x_dist * best_orthogonal_direction.0 + y_dist * best_orthogonal_direction.1;
        let dot_diagonal  =  x_dist * diag_direction.0 + y_dist * diag_direction.1;

        
        if dot_diagonal > 2*dot_orthogonal && !blocked.contains(&best_diagonal_tile) {
            return best_diagonal_tile
        } else if !blocked.contains(&best_orthogonal_tile) {
            return best_orthogonal_tile
        }

        // 4) look at all unblocked tiles that decrease either x_dist or y_dist and pick best by alignment.
        let direction_vectors = [
            (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1), (0, 1)
        ];

        let correct_new_directions = 
        direction_vectors.iter()
                    .filter(|(x, y)|
                    (*x == x_dist.signum() || *y == y_dist.signum())                                // x OR y is in correct direction
                    && ![diag_direction, best_orthogonal_direction].contains(&(*x, *y))             // and hasnt been checked already
                    && {
                        let dest = &self.translated_by((*x, *y));                                                  // visualise destination
                        0 <= dest.x && 0 <= dest.y && dest.x <= 9*LOGICAL_SUBTILES && dest.y <= 4*LOGICAL_SUBTILES // and destination isn't out of bounds
                        && !blocked.contains(&self.translated_by((*x, *y)))                                        // and destination isnt blocked
                    }); 

        //println!("{:?}", correct_new_directions.clone().collect::<Vec<_>>());
        
        let mut dirs_and_dots = correct_new_directions
                            .map(|(x, y)| {        
            ((*x, *y), (*x * x_dist + *y * y_dist)/(x.abs() + y.abs()))}) // this horrific thing computes the dot product and returns (coords, dot)
                            .filter(|((_x, _y), dot)| *dot > 0) // filter out tiles with negative results
                            .collect::<Vec<_>>();

        dirs_and_dots.sort_by_key(|((_x, _y), dot)| -dot); // sort largest to smallest
    
        // extract best direction. If none are found, best direction is (0, 0) i.e not moving.
        let best_dir = dirs_and_dots.iter().next().unwrap_or(&((0, 0), 0)).0; 
        self.translated_by(best_dir)
        

        // unimplemented!("both good tiles ({} and {}) are blocked. Implement better pathing\n {} -> {} aka \n{:?} -> {:?}\n {:?}",  best_orthogonal_tile, best_diagonal_tile, self, other, self, other, blocked);
    }
    */