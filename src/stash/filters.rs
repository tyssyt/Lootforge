use crate::{item::tags::Rating, mods::attune::Attunement, prelude::*};
use std::{collections::BTreeMap, ops::RangeInclusive, sync::atomic::{AtomicU32, Ordering}};

use enumset::EnumSet;

use crate::item::{item::Item, item_type::ItemType};

static ID_COUNTER: AtomicU32 = AtomicU32::new(1);

#[apply(Default)]
pub struct ItemFilter {
    types: EnumSet<ItemType>,
    #[default(EnumSet::all() - Rating::Trash)]
    rating: EnumSet<Rating>,
    #[default(1..=u8::MAX)]
    ranks: RangeInclusive<u8>,
    mods: BTreeMap<u16, u8>,
    attunement: Vec<Attunement>,
    excluded_item_ids: Vec<usize>,

    #[default(ID_COUNTER.fetch_add(1, Ordering::Relaxed))]
    id: u32,
    mod_count: u32,
}
impl ItemFilter {
    pub fn new(item_type: ItemType, rating: EnumSet<Rating>, rank: u8, mods: impl Iterator<Item = (u16, u8)>, excluded_item_ids: impl IntoIterator<Item = usize>) -> Self {
        Self {
            types: EnumSet::only(item_type),
            rating,
            ranks: rank..=rank,
            mods: mods.into_iter().collect(),
            attunement: Default::default(),
            excluded_item_ids: excluded_item_ids.into_iter().collect(),
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            mod_count: 0
        }
    }
    pub fn of_attunement(attunement: Vec<Attunement>, rating: EnumSet<Rating>, excluded_item_ids: impl IntoIterator<Item = usize>) -> Self {
        Self {
            attunement,
            rating,
            excluded_item_ids: excluded_item_ids.into_iter().collect(),
            ..Default::default()
        }
    }

    pub fn cache_key(&self) -> u64 {
        ((self.mod_count as u64) << 32) | (self.id as u64) 
    }

    pub fn clear(&mut self) {
        *self = ItemFilter {
            mod_count: self.mod_count + 1,
            ..Default::default()
        }
    }

    pub fn has_type(&self, item_type: ItemType) -> bool {
        self.types.contains(item_type)
    }
    pub fn toggle_type(&mut self, item_type: ItemType) {
        self.types ^= item_type;
        self.mod_count += 1;
    }

    pub fn has_rating(&self, rating: Rating) -> bool {
        self.rating.contains(rating)
    }
    pub fn toggle_rating(&mut self, rating: Rating) {
        self.rating ^= rating;
        self.mod_count += 1;
    }



    pub fn ranks(&self) -> RangeInclusive<u8> {
        self.ranks.clone()
    }
    pub fn set_ranks(&mut self, ranks: RangeInclusive<u8>) {
        if ranks.start() == self.ranks.start() && ranks.end() == self.ranks.end() {
            return;
        }
        self.ranks = ranks;
        self.mod_count += 1;
    }

    pub fn has_mod(&self, id: u16, count: u8) -> bool {
        self.mods.get(&id).is_some_and(|c| *c >= count) 
    }
    pub fn add_mod(&mut self, id: u16, count: u8) {
        if self.mods.insert(id, count).is_none_or(|c| c != count) {
            self.mod_count += 1;
        }
    }
    pub fn remove_mod(&mut self, id: u16) {
        if self.mods.remove(&id).is_some() {
            self.mod_count += 1;
        }
    }

    pub fn filter(&self, item: &Item) -> bool {
        (self.types.is_empty() || self.types.contains(item.item_type))
            && self.rating.contains(item.tags.rating())
            && self.ranks.contains(&item.rank())
            && item.has_all_mods(self.mods.iter().map(|(&m, &c)| (m, c)))
            && (self.attunement.is_empty() || item.attunements.iter().any(|a| self.attunement.contains(a)))
            && !self.excluded_item_ids.contains(&item.id)
    }
}