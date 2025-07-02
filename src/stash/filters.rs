use std::{collections::BTreeMap, ops::RangeInclusive, sync::atomic::{AtomicU32, Ordering}};

use enumset::EnumSet;

use crate::item::{Item, ItemType};

static ID_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct ItemFilter {
    types: EnumSet<ItemType>,
    ranks: RangeInclusive<u8>,
    mods: BTreeMap<u16, u8>,
    excluded_item_ids: Vec<usize>,

    id: u32,
    mod_count: u32,
}
impl Default for ItemFilter {
    fn default() -> Self {
        Self { 
            types: Default::default(),
            ranks: RangeInclusive::new(1, u8::MAX),
            mods: Default::default(),
            excluded_item_ids: Default::default(),
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            mod_count: Default::default()
        }
    }
}
impl ItemFilter {
    pub fn new(item_type: ItemType, rank: u8, mods: impl Iterator<Item = (u16, u8)>, excluded_item_ids: impl IntoIterator<Item = usize>) -> Self {
        Self {
            types: EnumSet::only(item_type),
            ranks: rank..=rank,
            mods: mods.into_iter().collect(),
            excluded_item_ids: excluded_item_ids.into_iter().collect(),
            id: ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            mod_count: 0
        }
    }

    pub fn cache_key(&self) -> u64 {
        ((self.mod_count as u64) << 32) | (self.id as u64) 
    }

    pub fn clear(&mut self) {
        self.types = Default::default();
        self.ranks = RangeInclusive::new(1, u8::MAX);
        self.mods = Default::default();
        self.mod_count += 1;
    }

    pub fn has_type(&self, item_type: ItemType) -> bool {
        self.types.contains(item_type)
    }
    pub fn toggle_type(&mut self, item_type: ItemType) {
        self.types ^= item_type;
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
            && self.ranks.contains(&item.rank())
            && item.has_all_mods(self.mods.iter().map(|(&m, &c)| (m, c)))
            && !self.excluded_item_ids.contains(&item.id)
    }
}