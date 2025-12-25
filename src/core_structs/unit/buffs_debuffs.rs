use crate::core_structs::unit::prelude::*;
use BuffEffect::*;

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum BuffEffect {
    AttackDamageModifier(AttackDamage),
    FlatIncomingReduction(Hitpoints)
}

impl BuffEffect{
    pub(crate) fn get_magnitude(&self) -> u64 {
        match self {
            AttackDamageModifier(n) => n.0,
            FlatIncomingReduction(n) => n.0,
        }
    }
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Buff {
    pub(crate) id: BuffID,
    pub(crate) buff_type: BuffEffect,
    pub(crate) priority: i8,           // for sequencing.
    pub(crate) counter: Option<u32>,   // either ticks remaining or activation instances (hits blocked, attacks empowered etc) remaining, depending on the buff. If None, permanent.
    pub(crate) max_stacks: Option<u8>  // guess most buffs will be 1. None means infinite
}

impl Buff {
    pub(crate) fn modify_damage(&self, damage: Hitpoints) -> Hitpoints {
        match self.buff_type {
            FlatIncomingReduction(n) => { Hitpoints(damage.0.saturating_sub(n.0)) }
            _ => damage
        }
    }
}

#[derive(Hash, Clone, PartialEq, Eq, Debug)]
pub struct BuffRing {
    // ring buffer for BuffEffects.  Ensures most recent buff effects are measured in buffs with non-1, non-infinite stack limits.
    buffer: Vec<Option<u64>>,
    capacity: usize,
    next_write: usize, //index to write to next
}

impl BuffRing  {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: (0..capacity).map(|_| None).collect(),
            capacity,
            next_write: 0,
        }
    }

    fn insert<T: Into<u64>>(&mut self, item: T) {
        self.buffer[self.next_write] = Some(item.into());
        self.next_write = (self.next_write + 1) % self.capacity
    }

    fn get_magnitude(&self) -> u64 {
        self.buffer.iter().filter_map(|opt|opt.as_ref().cloned()).sum()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BuffContainer{
    // contains everything to track an instance (or multiple if you count stacks seperately) of a buff in active play.
    pub(crate) buff: Buff,
    pub(crate) current_magnitude: u64,
    pub(crate) tracked_stacks: Option<BuffRing>
}

impl BuffContainer {
    pub(crate) fn new_from(buff: Buff) -> Self {
        let tracked_stacks = if let Some(max) = buff.max_stacks {
            let mut ring = BuffRing::with_capacity(max.into());
            ring.insert(buff.buff_type.get_magnitude());
            Some(ring)
        } else {None};

        BuffContainer {
            buff,
            current_magnitude: buff.buff_type.get_magnitude(),
            tracked_stacks
        }
    }

    pub(crate) fn add_stack(&mut self, magnitude: u64) {
        match &mut self.tracked_stacks {
            Some(ring) => {
                ring.insert(magnitude);
                self.current_magnitude = ring.get_magnitude();
            }
            None => {
                self.current_magnitude += magnitude;
            }
        }
    }

    pub(crate) fn to_single(&self) -> Buff {
        let mut out = Buff {
            ..self.buff
        };
        out.buff_type = match self.buff.buff_type {
            AttackDamageModifier(_n)  => AttackDamageModifier(AttackDamage(self.current_magnitude)),
            FlatIncomingReduction(_n) => FlatIncomingReduction(Hitpoints(self.current_magnitude))
        };
        out
    }
}