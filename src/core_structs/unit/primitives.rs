//! file contains the most basic data type wrappers for unit data.
use std::ops::{Add, AddAssign, Sub, SubAssign};

use rand::Rng;
use fixedstr::str32;

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct AttackDamage   (pub(crate) u64);
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Hitpoints      (pub(crate) u64);
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Shield         (pub(crate) Option<Hitpoints>); // hp shield, hit before hp
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Mitigation     (pub(crate) u32);  // serves as both Defence and Magic Resist
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct AttackTickDelay(pub(crate) u8);   // not likely to be above 255
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct CritChance     (pub(crate) u16);  // 0-65535, crits roll for rand under this.
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Mana           (pub(crate) u16);  // used for hash lookup#[derive(Hash, Clone, Copy, PartialEq, Eq)]
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub enum DamageType { Physical, Magic } // expand for more interesting synergies?#[derive(Hash, Clone, Copy, PartialEq, Eq)]

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct AttackRange    (pub(crate) i32);  // tiles, used in unit definitions (UnitTemplate)
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct SquaredLogicalRange(pub(crate) i32);  // logical subtiles, used by BattleUnit in actual combat.

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct UnitTemplateID (pub(crate) u16, pub(crate) Roster); // used for hash lookup
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct BuffID         (pub(crate) u16); 
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct EntityID       (pub(crate) u16, pub(crate) u16);       

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub enum Team   { Player, Opponent } 
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub enum Roster { Human, NPC } 

#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct BattleSubtile {pub x: i32, pub y: i32} // logical position including subtiles.
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct GridPosition {pub x: i32, pub y: i32} // logical position including subtiles.

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]                pub struct SecondsPerTile (pub(crate) f32);  // limited rights because of f32
#[derive(Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct MoveSpeed      (pub(crate) i32);  // used for hash lookup#[derive(Hash, Clone, Copy, PartialEq, Eq)]


pub(crate) use DamageType::*;

use crate::core_structs::unit::{data::{
    enemies::ENEMY_DATABASE, roster::UNIT_DATABASE
}, spatial_functions::{LOGICAL_SUBTILES, TICKS_PER_SECOND}};

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
        Self((perc/100.0 * u16::MAX as f32) as u16)
    }

    pub fn to_percentage(&self) -> f32 {
        self.0 as f32 / u16::MAX as f32
    }

    pub fn did_crit<T: Rng>(self, rng: &mut T) -> bool {
        let rand = rng.random();
        // //println!("crit roll: {}/65535 - threshold {}", rand, self.0);
        self.0 >= rand
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

impl SecondsPerTile {
    pub fn to_logical(&self) -> MoveSpeed {
        let tiles_per_second = 1.0/self.0;
        let tiles_per_tick = tiles_per_second/TICKS_PER_SECOND as f32;
        let subtiles_per_tick = tiles_per_tick*LOGICAL_SUBTILES as f32;

        #[cfg(test)]
        println!("Movement speed: {}s/tile converted to {} subtiles/tick", &self.0, &subtiles_per_tick);
        
        MoveSpeed(subtiles_per_tick as i32)
    }
}