use rand::distr::weighted::WeightedIndex;

use crate::prelude::*;

use crate::item::{item::Item, item_type::ItemType};

#[derive(Debug)]
pub struct RewardChest {
    pub depth: u16,
    pub items: Vec<Item>,
}
impl RewardChest {

    pub fn from(rng: &mut impl Rng, depth: u16) -> Self {
        if depth == 0 {
            return Self { depth, items: Vec::new() };
        }        

        let count = count(rng, depth);
        let max_rank = (depth + 10) / 10;
        let overrank_chance = overrank_chance(depth);
        let item_types = item_types(depth);

        let mut items = bonus_items(rng, max_rank, overrank_chance, item_types, count-1);
        items.push(max_item(rng, max_rank, overrank_chance, item_types));
        Self { depth, items }
    }
}

fn count(rng: &mut impl Rng, depth: u16) -> u16 {
    if depth < 5 {
        return 1;
    }

    let base = (depth+15) / 10;
    let bonus_chance = (depth+15) % 10;
    if rng.random_bool((bonus_chance as f64) / 10.) {
        base + 1
    } else {
        base
    }
}
fn overrank_chance(depth: u16) -> f64 {
    if depth < 5 {
        0.
    } else {
        ((depth % 10) as f64) / 10.0
    }
}
fn item_types(depth: u16) -> &'static[ItemType] {
    match depth {
        0 => &[ItemType::Axe, ItemType::Armor], // should not happen
        1 => &[ItemType::Axe, ItemType::Armor, ItemType::Helmet, ItemType::Shield],
        2 => &[ItemType::Axe, ItemType::Armor, ItemType::Helmet, ItemType::Shield, ItemType::Gloves],
        3 => &[ItemType::Axe, ItemType::Armor, ItemType::Helmet, ItemType::Shield, ItemType::Gloves, ItemType::Ring],
        4..20 => &[ItemType::Axe, ItemType::Armor, ItemType::Helmet, ItemType::Shield, ItemType::Gloves, ItemType::Ring, ItemType::Sword],
        // 20..MAGE warrior & ranger items
        _ => ItemType::VARIANTS,
    }
}

fn bonus_items(rng: &mut impl Rng, max_rank: u16, overrank_chance: f64, item_types: &'static[ItemType], count: u16) -> Vec<Item> {
    let mut weights = vec![1.0; max_rank as usize];
    weights.push(overrank_chance);

    let dist = WeightedIndex::new(&weights).unwrap();

    repeat_n((), count as usize)
        .map(|_| bonus_item(rng, &dist, item_types))
        .collect()
}

fn bonus_item(rng: &mut impl Rng, dist: &WeightedIndex<f64>, item_types: &'static[ItemType]) -> Item {
    let item_type = *item_types.pick(rng);
    let rank = (dist.sample(rng) as u8 +1).try_into().expect("congrats, you broke the game");
    Item::random(rng, item_type, rank)
}

fn max_item(rng: &mut impl Rng, max_rank: u16, overrank_chance: f64, item_types: &'static[ItemType]) -> Item {
    let item_type = *item_types.pick(rng);  
    if rng.random_bool(overrank_chance) {
        Item::random(rng, item_type, max_rank as u8 +1)
    } else {
        Item::random(rng, item_type, max_rank as u8)
    }
}