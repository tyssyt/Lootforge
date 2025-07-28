use std::iter;

use enumset::EnumSet;

use crate::item::tags::Rating;
use crate::prelude::*;
use super::common::*;

use crate::item::{item::Item, item::ItemRef};
use crate::stash::filters::ItemFilter;
use crate::stash::stash::Stash;


#[apply(Default)]
pub struct RerollTarget {
    selected_mod: Option<u8>,
    material: ItemRef,
}

impl RerollTarget {
    fn selected_mod(&self, base: &Item) -> Option<u16> {
        self.selected_mod.map(|i| base[i].mod_id)
    }
    fn valid_mat(&self, mat: &Item, base: &Item) -> bool {
        mat != base && mat.item_type == base.item_type && mat.rank() == base.rank()
            && self.selected_mod(base).is_some_and(|m| mat.has_mod(m))
    }

    pub fn show(&mut self, base_ref: &mut ItemRef, ui: &mut Ui, stash: &mut Stash) -> bool {
        // more styling and stuff
        ui.label("Reroll a targeted modifier. This requires an item of the same rank with the targeted mod as material");
        ui.label("Once you target reroll a modifier, it is marked as rerolled, and no other mods can be targeted for this forge");

        ui.add_space(8.);
        let mut changed = false;

        if let Some(base) = base_ref.upgrade() {
            if let Some(i) = base.rerolled_mod_idx() {
                self.selected_mod = Some(i)
            }

            ui.horizontal_top(|ui| {
                changed |= show_item_slot(base_ref, vec2(64., 64.), None, ui, stash, accepts_base);
                ui.vertical(|ui| {
                    let enabled = base
                        .rerolled_mod_idx()
                        .map(|i| vec![i])
                        .unwrap_or_else(|| (0..base.rank()).collect());
                    changed |= show_mod_table_single(ui, &base, &mut self.selected_mod, enabled);
                });
            });

            if self.material.upgrade().is_some_and(|mat| !self.valid_mat(&mat, &base)) {
                self.material = ItemRef::new();
            }

            ui.add_space(8.);

            { // TODO I don't like this work around, and I bet there is a nicer way to do this in rust...
                let mut mat = self.material.clone();
                changed |= show_item_slot(&mut mat, vec2(32., 32.), Some(base.item_type), ui, stash, |mat| self.valid_mat(mat, &base));
                self.material = mat;
            }

            ui.add_space(8.);

            let enabled = self.material.upgrade().is_some();
            if show_forge_button(enabled, ui).clicked() {
                self.forge(base, stash);
                changed = true;
            }
        } else {
            ui.vertical_centered(|ui| {
                changed |= show_item_slot(base_ref, vec2(64., 64.), None, ui, stash, accepts_base);
            });
        }
        changed
    }

    fn forge(&mut self, base: Rc<Item>, stash: &mut Stash) {
        let mut rng = rand::rng();
        let idx = self.selected_mod.unwrap();
        let old_mod_id = base[idx].mod_id;

        let new_mod = loop {
            let new_mod = base.item_type.roll_mod(&mut rng, &base.mods);
            if new_mod.mod_id != old_mod_id {
                break new_mod;
            }
        };

        stash.remove(self.material.upgrade().unwrap());
        stash.modify(base, |base| {
            base.mods[idx as usize] = new_mod;
            base.rerolled_mod_idx = idx;
        });
    }

    pub fn filter(&self, base: &Item) -> ItemFilter {
        let mods = self.selected_mod(base).into_iter().map(|m| (m,1));
        let excluded = iter::once(base.id).chain(
            self.material.upgrade().map(|item| item.id)
        );

        ItemFilter::new(
            base.item_type,
            EnumSet::all() - Rating::Favorite,
            base.rank(), mods,
            excluded
        )
    }
}

fn accepts_base(_item: &Item) -> bool {
    true
}