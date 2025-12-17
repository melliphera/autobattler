//! file contains the most basic data type wrappers for unit data.
use std::ops::{Add, AddAssign, Sub, SubAssign};

use rand::Rng;
use fixedstr::str32;

pub static LOGICAL_SUBTILES: i32 = 4096;

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct AttackDamage   (pub(crate) u64);
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct Hitpoints      (pub(crate) u64);
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct Shield         (pub(crate) Option<Hitpoints>); // hp shield, hit before hp
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct Mitigation     (pub(crate) u32);  // serves as both Defence and Magic Resist
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct AttackTickDelay(pub(crate) u8);   // not likely to be above 255
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct CritChance     (pub(crate) u16);  // 0-65535, crits roll for rand under this.
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct AttackRange    (pub(crate) i32);  // tiles - fast conversion to f32
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct Mana           (pub(crate) u16);  // used for hash lookup#[derive(Hash, Clone, Copy, PartialEq, Eq)]
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) enum DamageType { Physical, Magic } // expand for more interesting synergies?#[derive(Hash, Clone, Copy, PartialEq, Eq)]

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct UnitTemplateID (pub(crate) u16, pub(crate) Roster); // used for hash lookup
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct BuffID         (pub(crate) u16); 
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct EntityID       (pub(crate) u8);       

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) enum Team   { Player, Opponent } 
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) enum Roster { Human, NPC } 

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct BattlePosition {pub(crate) x: i32, pub(crate) y: i32}
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub(crate) struct MoveSpeed      (pub(crate) i32);  // used for hash lookup#[derive(Hash, Clone, Copy, PartialEq, Eq)]


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

    pub(crate) fn did_crit<T: Rng>(self, rng: &mut T) -> bool {
        let rand = rng.random();
        // println!("crit roll: {}/65535 - threshold {}", rand, self.0);
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
        AttackRange((dx*dx + dy*dy))
    }

    pub(crate) fn to_logical(self) -> BattlePosition {
        BattlePosition { x: self.x * LOGICAL_SUBTILES, y: self.y * LOGICAL_SUBTILES }
    }

    pub(crate) fn best_next_tile(&self, other: &Self, target_distance: AttackRange) -> BattlePosition {
        //! Chooses an adjacent (diagonal included) tile for a MoveEvent based on the following criteria:
        //! 0) if in same row or col as target, move closer along row or col.
        //! otherwise:
        //! 1) if best orthogonal tile puts within target_distance, do that
        //! 2) if best diagonal tile puts within target_distance, do that instead
        //! 3) if neither do, move in direction most closely aligned with target direction.
        //! 
        //! Diagonal is deliberately deprioritised via steps 2/3 because it takes 41% longer to walk there.
         
        // 0 
        if self.x == other.x {
            if self.y < other.y { 
                BattlePosition{ x: self.x, y: self.y+LOGICAL_SUBTILES }
            } else{
                BattlePosition{ x: self.x, y: self.y-LOGICAL_SUBTILES }
            }
        } else if self.y == other.y {
            if self.x < other.x {
                BattlePosition{ x: self.x+LOGICAL_SUBTILES, y: self.y }
            } else {
                BattlePosition{ x: self.x-LOGICAL_SUBTILES, y: self.y }
            }
        } else {

            // 1) check if orthogonal brings into range
            let (x_dist, y_dist) = (other.x - self.x, other.y - self.y);
            let best_orthogonal_direction: (i32, i32);
            if x_dist == y_dist {
                best_orthogonal_direction = (x_dist.signum(), 0);
            } else {
                let normaliser = x_dist.abs().max(y_dist.abs());
                best_orthogonal_direction = (x_dist/normaliser, y_dist/normaliser); // because of floor division this returns either (+-1, 0) or (0, +-1) unless x=y as handled above
            }
            let best_orthogonal_tile = BattlePosition{
                x: self.x + best_orthogonal_direction.0*LOGICAL_SUBTILES, 
                y: self.y + best_orthogonal_direction.1*LOGICAL_SUBTILES
            };
            if best_orthogonal_tile.distance_squared_to(other) <= target_distance {
                return best_orthogonal_tile;
            }

            // 2) check if diagonal brings into range
            let diag_direction = (x_dist.signum(), y_dist.signum());
            let best_diagonal_tile = BattlePosition {
                x: self.x + diag_direction.0*LOGICAL_SUBTILES,
                y: self.y + diag_direction.0*LOGICAL_SUBTILES
            };
            if best_diagonal_tile.distance_squared_to(other) <= target_distance {
                return best_diagonal_tile;
            }
            // 3) use dot product to see if orthogonal or diagonal is better
            let dot_orthogonal = x_dist * best_orthogonal_direction.0 + y_dist * best_orthogonal_direction.1;
            let dot_diagonal  =  x_dist * diag_direction.0 + y_dist * diag_direction.1;

            if dot_diagonal > 2*dot_orthogonal {
                best_diagonal_tile
            } else {
                best_orthogonal_tile
            }
        }
    }
}

impl UnitTemplateID {
    pub(crate) fn get_name(self) -> str32 {
        match self.1 {
            Roster::Human => {UNIT_DATABASE[self.0 as usize].name}
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
