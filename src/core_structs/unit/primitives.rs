//! file contains the most basic data type wrappers for unit data.
use std::ops::{Add, AddAssign, Sub, SubAssign};

use rand::Rng;
use fixedstr::str32;

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct AttackDamage   (pub u64);
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Hitpoints      (pub u64);
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Shield         (pub Option<Hitpoints>); // hp shield, hit before hp
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Mitigation     (pub u32); // serves as both Defence and Magic Resist
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct AttackTickDelay(pub u8);  // not likely to be above 255
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct CritChance     (pub u16); // 0-65535, crits roll for rand under this.
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct AttackRange    (pub i32);  // tiles - fast conversion to f32
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub enum DamageType { Physical, Magic } // expand for more interesting synergies?#[derive(Hash, Clone, Copy, PartialEq, Eq)]
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct UnitTemplateID (pub u16, pub Roster); // used for hash lookup
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct EntityID (pub u8);        // used for hash lookup#[derive(Hash, Clone, Copy, PartialEq, Eq)]
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub enum Team   { Player, Opponent } 
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub enum Roster { Human, NPC } 
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct BattlePosition {pub x: i32, pub y: i32}

pub use DamageType::*;

use crate::core_structs::unit::{
    roster::UNIT_DATABASE,
    enemies::ENEMY_DATABASE
};

impl AttackDamage {
    pub fn add(self, other: &Self) -> Self {
        // for adding e.g weapon stats
        AttackDamage(self.0 + other.0)
    }

    pub fn multiply(self, other: f32) -> Self {
        // for eg temporary buffs
        AttackDamage((self.0 as f32 * other) as u64)
    }
}

impl AttackTickDelay {
    pub fn get_attack_speed(self, ticks_per_sec: u8) -> f32 {
        // converts tick delay into a conventional attack speed based on tps. Used for display purposes.
        (ticks_per_sec as f32) / (self.0 as f32)
    }
}

impl CritChance {
    pub const fn from_percentage(perc: f32) -> Self {
        Self((perc * u16::MAX as f32) as u16)
    }

    pub fn did_crit<T: Rng>(self, rng: &mut T) -> bool {
        self.0 >= rng.random()
    }
}

impl AttackRange {
    pub fn to_squared(self) -> Self {
        AttackRange((self.0*64).pow(2)) //  64 units per tile
    }
}

impl BattlePosition {
    pub fn distance_squared_to(&self, other: &Self) -> AttackRange {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        AttackRange(dx*dx + dy*dy)
    }
}

impl UnitTemplateID {
    pub fn get_name(self) -> str32 {
        match self.1 {
            Roster::Human => {UNIT_DATABASE[self.0 as usize].name}
            Roster::NPC   => {ENEMY_DATABASE[self.0 as usize].name}
        }
    }
}

impl Team {
    pub fn opponent(self) -> Team {
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