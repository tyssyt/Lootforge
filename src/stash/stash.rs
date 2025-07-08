use crate::{item::ItemType, mods::{atk_mod, char_mod, RolledMod}, prelude::*, stash::{filters::ItemFilter, order::Order}};
use std::mem;

use crate::item::Item;


#[derive(PartialEq)]
struct FilterCacheKey(u64, Order);

#[derive(derive_more::Debug, SmartDefault)]
pub struct Stash {
    items: Vec<Rc<Item>>,
    #[default(1)]
    next_id: usize,
    #[default(1)]
    max_rank: u8,
    #[debug(skip)]
    cached_filter: Option<(FilterCacheKey, Rc<Vec<Rc<Item>>>)>, // could add a refcell so I don't need to pass mut stash around
}

impl Stash {
    fn get_next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn max_rank(&self) -> u8 {
        self.max_rank
    }

    pub fn add(&mut self, mut item: Item) {
        self.cached_filter = None;
        self.max_rank = item.rank().at_least(self.max_rank);

        item.id = self.get_next_id();
        self.items.push(Rc::new(item));
    }

    pub fn remove(&mut self, item: Rc<Item>) {
        self.cached_filter = None;
        self.items.remove(self.items.iter().position(|i| *i == item).unwrap());
        let count = Rc::strong_count(&item);
        if count != 1 {
            panic!("Attempting to delete item {}, but count is {}", item.id, count);
        }
    }

    pub fn modify(&mut self, item: Rc<Item>, f: impl FnOnce(&mut Item)) {
        self.cached_filter = None;

        let id = item.id;
        mem::drop(item); // invalidate the rc, so that the only remaining one is the stashes

        let item = self.items.iter_mut().find(|item| item.id == id).unwrap();
        let count = Rc::strong_count(&item);
        if count != 1 {
            panic!("Attempting to modify item {}, but count is {}", item.id, count)
        }

        unsafe {            
            // Rc::get_mut_unchecked
            let ptr = Rc::as_ptr(item) as *mut Item;
            f(&mut *ptr);
        }
    }

    pub fn find(&self, item_id: usize) -> Option<Rc<Item>> {
        self.items
            .iter()
            .find(|item| item.id == item_id)
            .map(|item| item.clone())
    }

    pub fn items(&self) -> &Vec<Rc<Item>> {
        &self.items
    }

    pub fn filtered_items(&mut self, filter: &ItemFilter, order: Order) -> Rc<Vec<Rc<Item>>> { // TODO I could create copies,,,
        let cache_key = FilterCacheKey(filter.cache_key(), order);
        if self.cached_filter.as_ref().is_some_and(|(ck, _)| cache_key == *ck) {
            return self.cached_filter.as_ref().unwrap().1.clone();
        }

        let filtered = Rc::new(self.items.iter()
            .filter(|item| filter.filter(item))
            .map(|item| item.clone())
            .sorted_by(|a, b| order.cmp(a, b))
            .collect::<Vec<_>>());

        self.cached_filter = Some((cache_key, filtered.clone()));
        filtered
    }

    pub fn give_starting_items(&mut self) {
        let start_weapon = Item::new(
            ItemType::Axe,
            vec![RolledMod { mod_id: atk_mod::ADDED_DMG.bleed.id, roll: *atk_mod::ADDED_DMG.bleed.roll_range.start() }],
            None
        );
        self.add(start_weapon);

        let start_armor = Item::new(
            ItemType::Armor,
            vec![RolledMod { mod_id: char_mod::HEALTH.id, roll: *char_mod::HEALTH.roll_range.start() }],
            None
        );
        self.add(start_armor);
        
        // let test = Item::new(
        //     ItemType::Axe,
        //     vec![
        //         RolledMod { mod_id: atk_mod::ADDED_DMG.bleed.id, roll: 10 },
        //         RolledMod { mod_id: atk_mod::DEBUFF_OFF_ST.id, roll: 0 },
        //     ],
        //     None
        // );
        // self.add(test);
    }
}
