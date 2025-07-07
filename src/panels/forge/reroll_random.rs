use std::iter;

use crate::prelude::*;
use super::common::*;

use crate::item::{Item, ItemRef};
use crate::stash::filters::ItemFilter;
use crate::stash::stash::Stash;


#[apply(Default)]
pub struct RerollRandom {
    selected_mods: Vec<usize>,
    material: ItemRef,
}

impl RerollRandom {
    fn selected_mod_counts(&self, base: &Item) -> impl Iterator<Item = (u16, u8)> {
        self.selected_mods.iter()
            .map(|i| base.mods[*i].mod_id)
            .counts().into_iter()
            .map(|(m, c)| (m, c as u8))
    }

    pub fn show(&mut self, base_ref: &mut ItemRef, ui: &mut Ui, stash: &mut Stash) {
        // more styling and stuff
        ui.label("Reroll a random modifier. This requires an item of the same rank as material");
        ui.label("If a modifier is shared between the items, it is protected from rerolling");

        ui.add_space(8.);

        if let Some(base) = base_ref.upgrade() {
            // toggle button that filters stash for right items

            ui.horizontal_top(|ui| {
                show_item_slot(base_ref, vec2(64., 64.), None, ui, stash, accepts_base);
                ui.vertical(|ui| show_mod_table(ui, &base, &mut self.selected_mods));
            });

            if self.material.upgrade().is_some_and(|mat| !mat.has_all_mods(self.selected_mod_counts(&base))) {
                self.material = ItemRef::new();
            }

            ui.add_space(8.);

            show_item_slot(&mut self.material, vec2(32., 32.), Some(base.item_type), ui, stash, |mat| accepts_mat(mat, &base),);

            ui.add_space(8.);

            let enabled = self.material.upgrade().is_some() && self.unprotected_mods(&base).len() > 0;
            if show_forge_button(enabled, ui).clicked() {
                self.forge(base, stash);
            }
        } else {
            ui.vertical_centered(|ui| {
                show_item_slot(base_ref, vec2(64., 64.), None, ui, stash, accepts_base);
            });
        }
    }

    fn forge(&mut self, base: Rc<Item>, stash: &mut Stash) {
        let mut rng = rand::rng();
        let unprotected_mods = self.unprotected_mods(&base);
        let old_mod_id = unprotected_mods.choose_weighted(&mut rng, |(_,c)| *c).unwrap().0;
        let old_mod_idx = base.mods.iter().enumerate().filter(|(_, m)| m.mod_id == old_mod_id).choose(&mut rng).unwrap().0;

        let new_mod = loop {
            let new_mod = base.item_type.roll_mod(&mut rng, &base.mods);
            if new_mod.mod_id != old_mod_id {
                break new_mod;
            }
        };

        stash.remove(self.material.upgrade().unwrap());        
        unsafe {
            let id = base.id;
            let item = stash
                .get_mut(base)
                .expect(&format!("Could not get a mutable ref to item: {}", id));

            item.mods[old_mod_idx] = new_mod;
        }
    }

    fn unprotected_mods(&self, base: &Item) -> Vec<(u16, u8)> {  
        let material = self.material.upgrade().unwrap();
        base.mods.iter()
            .map(|m| m.mod_id)
            .counts().into_iter()
            .map(|(m, c)| (m, c as u8))
            .map(|(m, c)| (m, c - material.has_mod(m).at_most(c)))
            .filter(|(_, c)| *c > 0)
            .collect()
    }

    pub fn filter(&self, base: &Item) -> ItemFilter {
        let excluded = iter::once(base.id).chain(
            self.material.upgrade().map(|item| item.id)
        );

        ItemFilter::new(base.item_type, base.rank(), self.selected_mod_counts(base), excluded)
    }
}

fn accepts_base(_item: &Item) -> bool {
    true
}
fn accepts_mat(mat: &Item, base: &Item) -> bool {
    mat != base && mat.item_type == base.item_type && mat.rank() == base.rank()
}