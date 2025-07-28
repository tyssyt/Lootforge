use crate::combat::combatant::CombatantKind;
use crate::combat::enemy::EnemyType;
use crate::prelude::*;

use crate::{combat::combatant::Combatant, elemental::Element};

use Targeting::*;

#[apply(Enum)]
#[derive(Copy, PartialEq)]
pub enum Targeting {
    // default
    First,

    // Rings
    LowestHealth,
    LowestResistance(Element),
    HighestMaxHealth,
    HighestDamage,
    RoundRobin(u8),
    // atk together with ally (attunable)
    // Fighter, Range, Mage (for support skills once we have those chars in)

    // Most Buffs, most Debuffs? also great for support skills

    // triggering for defense skills
    Instant,
    OnAttack,
}
impl Targeting {
    pub fn roll_ring(rng: &mut impl Rng) -> Self {
        match rng.random_range(0..=4) {
            0 => LowestHealth,
            1 => LowestResistance(*Element::VARIANTS.pick(rng)),
            2 => HighestMaxHealth,
            3 => HighestDamage,
            4 => RoundRobin(0),
            _ => panic!(),
        }
    }

    pub fn select_target<'a, 'b>(&mut self, targets: &'a mut Vec<&'b mut Combatant>) -> &'a mut Combatant {
        match self {
            First => targets[0],
            LowestHealth => targets.iter_mut().min_by_key(|t| F32Ord(t.health)).unwrap(),
            LowestResistance(element) => targets.iter_mut().min_by_key(|t| F32Ord(*t.stats().resistances.get(*element))).unwrap(), // TODO incredibly inefficient because char() does not get cached
            HighestMaxHealth => targets.iter_mut().max_by_key(|t| F32Ord(t.stats().max_health)).unwrap(),
            HighestDamage => {
                // TODO if we have stats, we can do this maybe better
                targets.iter_mut().max_by_key(|t| match t.kind {
                    CombatantKind::Fighter => panic!(),
                    CombatantKind::Enemy(_, enemy_kind) => match enemy_kind.etype() {
                        EnemyType::Small => 1,
                        EnemyType::Medium => 3,
                        EnemyType::Tank => 2,
                        EnemyType::Dps => 4,
                    },
                }).unwrap()
            },
            RoundRobin(i) => {
                let len = targets.len();
                *i = i.wrapping_add(1); targets[*i as usize % len]
            },
            Instant | OnAttack => panic!(),
        }
    }
}
