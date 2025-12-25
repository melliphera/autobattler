//! file contains the most basic data type wrappers for unit data.
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::fmt::Display;

use rand::Rng;
use fixedstr::str32;

pub static LOGICAL_SUBTILES: i32 = 4096;

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct AttackDamage   (pub(crate) u64);
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Hitpoints      (pub(crate) u64);
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Shield         (pub(crate) Option<Hitpoints>); // hp shield, hit before hp
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Mitigation     (pub(crate) u32);  // serves as both Defence and Magic Resist
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct AttackTickDelay(pub(crate) u8);   // not likely to be above 255
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct CritChance     (pub(crate) u16);  // 0-65535, crits roll for rand under this.
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct AttackRange    (pub(crate) i32);  // tiles - fast conversion to f32
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Mana           (pub(crate) u16);  // used for hash lookup#[derive(Hash, Clone, Copy, PartialEq, Eq)]
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub enum DamageType { Physical, Magic } // expand for more interesting synergies?#[derive(Hash, Clone, Copy, PartialEq, Eq)]

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct UnitTemplateID (pub(crate) u16, pub(crate) Roster); // used for hash lookup
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct BuffID         (pub(crate) u16); 
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct EntityID       (pub(crate) u16, pub(crate) u16);       

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub enum Team   { Player, Opponent } 
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub enum Roster { Human, NPC } 

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct BattlePosition {pub(crate) x: i32, pub(crate) y: i32}
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct MoveSpeed      (pub(crate) i32);  // used for hash lookup#[derive(Hash, Clone, Copy, PartialEq, Eq)]


pub(crate) use DamageType::*;

use crate::core_structs::unit::data::{
    roster::UNIT_DATABASE,
    enemies::ENEMY_DATABASE
};

impl AttackDamage {
    pub(crate) fn add(self, other: &Self) -> Self {
        // for adding e.g weapon stats
        AttackDamage(self.0 + other.0)
    }

    pub(crate) fn multiply(self, other: f32) -> Self {
        // for eg temporary buffs
        AttackDamage((self.0 as f32 * other) as u64)
    }
}

impl AttackTickDelay {
    pub(crate) fn get_attack_speed(self, ticks_per_sec: u8) -> f32 {
        // converts tick delay into a conventional attack speed based on tps. Used for display purposes.
        (ticks_per_sec as f32) / (self.0 as f32)
    }
}

impl CritChance {
    pub(crate) const fn from_percentage(perc: f32) -> Self {
        Self((perc/100.0 * u16::MAX as f32) as u16)
    }

    pub(crate) fn to_percentage(&self) -> f32 {
        self.0 as f32 / u16::MAX as f32
    }

    pub(crate) fn did_crit<T: Rng>(self, rng: &mut T) -> bool {
        let rand = rng.random();
        // //println!("crit roll: {}/65535 - threshold {}", rand, self.0);
        self.0 >= rand
    }
}

impl AttackRange {
    pub(crate) fn to_squared(self) -> Self {
        AttackRange((self.0*LOGICAL_SUBTILES).pow(2)) //  LOGICAL_SUBTILES "logical pixels" per tile
    }
}

impl BattlePosition {
    pub(crate) fn distance_squared_to(&self, other: &Self) -> AttackRange {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        AttackRange(dx*dx + dy*dy)
    }

    pub(crate) fn translated_by(&self, tuple: (i32, i32)) -> BattlePosition {
        BattlePosition { x: self.x + tuple.0*LOGICAL_SUBTILES, y: self.y + tuple.1*LOGICAL_SUBTILES }
    }


    pub(crate) fn to_logical(self) -> BattlePosition {
        BattlePosition { x: self.x * LOGICAL_SUBTILES, y: self.y * LOGICAL_SUBTILES }
    }

    pub(crate) fn best_next_tile(&self, other: &Self, target_distance: AttackRange, blocked: &Vec<BattlePosition>) -> BattlePosition {
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
        // 3) use dot product to see if orthogonal or diagonal is better assuming both unblocked
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

        let correct_new_directions = direction_vectors.iter()
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
}

impl Display for BattlePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x/LOGICAL_SUBTILES, self.y/LOGICAL_SUBTILES)
    }
    
}

impl UnitTemplateID {
    pub(crate) fn get_name(self) -> str32 {
        match self.1 {
            Roster::Human => {UNIT_DATABASE [self.0 as usize].name}
            Roster::NPC   => {ENEMY_DATABASE[self.0 as usize].name}
        }
    }
}

impl Team {
    pub(crate) fn opponent(self) -> Team {
        match self {
            Team::Player => Team::Opponent,
            Team::Opponent => Team::Player
        }
    }
}

impl Add for Hitpoints {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Hitpoints(self.0 + rhs.0)
    }
}

impl AddAssign for Hitpoints {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl Sub for Hitpoints {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        // halts at 0
        Hitpoints(self.0-rhs.0.min(self.0))
    }
}

impl SubAssign for Hitpoints {
    // halts at 0
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0.min(self.0)
    }
}

impl Mana {
    pub(crate) fn add(&mut self, to_add: u16, max_mana: Mana) {
        if self.0 + to_add <= max_mana.0 {
            self.0 = self.0 + to_add
        } else {
            self.0 = max_mana.0
        }
    }
}
