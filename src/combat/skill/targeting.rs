use crate::combat::skill::hit;
use crate::prelude::*;

use crate::{combat::combatant::Combatant, elemental::Element};

use Targeting::*;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Targeting {
    // default
    First,

    // Rings
    LowestHealth,
    LowestEffectiveHealth(Element),
    LowestResistance(Element),
    HighestRank, // or just dps
    LowestRank, // or just dps
    RoundRobin(u8),
    // atk together with ally (attunable)
    // Fighter, Range, Mage (for support skills once we have those chars in)

    // Most Buffs, most Debuffs? also great for support skills

    // triggering for defense skills
    Instant,
    OnAttack,
}
impl Targeting {
    pub fn roll_ring<R: Rng>(rng: &mut R) -> Self {
        match rng.random_range(0..4) {
            0 => LowestHealth,
            1 => LowestEffectiveHealth(*Element::VARIANTS.choose(rng).unwrap()),
            2 => LowestResistance(*Element::VARIANTS.choose(rng).unwrap()),
            3 => RoundRobin(0),
            _ => panic!(),
        }
    }

    pub fn select_target<'a, 'b>(&mut self, targets: &'a mut Vec<&'b mut Combatant>) -> &'a mut Combatant {
        match self {
            First => targets[0],
            LowestHealth => targets.iter_mut().min_by_key(|t| F32Ord(t.health)).unwrap(),
            LowestEffectiveHealth(element) => targets.iter_mut().max_by_key(|t| F32Ord(hit::effective_health(t.health, *t.stats().resistances.get(*element)))).unwrap(), // TODO incredibly inefficient because char() does not get cached
            LowestResistance(element) => targets.iter_mut().min_by_key(|t| F32Ord(*t.stats().resistances.get(*element))).unwrap(), // TODO incredibly inefficient because char() does not get cached
            HighestRank => todo!(),
            LowestRank => todo!(),
            RoundRobin(i) => {let len = targets.len(); *i += 1; targets[*i as usize % len]}, // TODO this behaves a bit weird when an enemy dies
            Instant | OnAttack => panic!(),
        }
    }
}
