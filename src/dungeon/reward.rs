use rand::distr::weighted::WeightedIndex;

use crate::prelude::*;

use crate::item::Item;

#[derive(Debug)]
pub struct RewardChest {
    pub items: Vec<Item>,
    // TODO a ref to the stats of the run
}
impl RewardChest {
    pub fn from(rng: &mut impl Rng, depth: u16) -> Self {
        if depth == 0 {
            return Self { items: Vec::new() };
        }

        let bonus_count = (depth + 5) / 10;
        let max_rank = (depth + 10) / 10;

        let overrank_chance = ((depth % 10) as f64) / 10.0;

        let mut weights = vec![1.0; max_rank as usize];
        weights.push(overrank_chance);

        let dist = WeightedIndex::new(&weights).unwrap();

        let mut items: Vec<_> = repeat_n((), bonus_count as usize)
            .map(|_| {
                let rank = (dist.sample(rng) as u8 +1).try_into().expect("congrats, you broke the game");
                Item::random(rng, rank)
            })
            .collect();

        let max_item = if rng.random_bool(overrank_chance) {
            Item::random(rng, max_rank as u8 +1)
        } else {
            Item::random(rng, max_rank as u8)
        };
        items.push(max_item);

        Self { items }
    }
}