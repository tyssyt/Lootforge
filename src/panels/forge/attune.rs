use std::iter;

use enumset::EnumSet;

use crate::item::tags::Rating;
use crate::mods::attune::AttuneGroup;
use crate::mods::RolledMod;
use crate::prelude::*;
use super::common::*;

use crate::item::{item::Item, item::ItemRef};
use crate::stash::filters::ItemFilter;
use crate::stash::stash::Stash;


#[apply(Default)]
pub struct Attune {
    selected_mod: Option<u8>,
    material: ItemRef,
}

impl Attune {
    fn selected_mod<'a>(&'a self, base: &'a Item) -> Option<&'a RolledMod> {
        self.selected_mod.map(|i| &base[i])
    }
    fn selected_attunement(&self, base: &Item) -> Option<(&'static AttuneGroup, usize)> {
        self.selected_mod(base).and_then(|m| m.mod_type().attunement())
    }
    fn valid_mat(&self, mat: &Item, base: &Item) -> bool {
        mat != base && self.selected_attunement(base).is_some_and(|(group, idx)| mat.attunements.iter().any(|(k, i)| group.kind == *k && idx != *i))
    }

    pub fn show(&mut self, base_ref: &mut ItemRef, ui: &mut Ui, stash: &mut Stash) -> bool {
        // more styling and stuff
        ui.label("Change the attunement of a modifier. It will gain the attunement of the material");
        ui.label("The material may not have conflicting attunements");

        ui.add_space(8.);
        let mut changed = false;

        if let Some(base) = base_ref.upgrade() {
            ui.horizontal_top(|ui| {
                changed |= show_item_slot(base_ref, vec2(64., 64.), None, ui, stash, accepts_base);
                ui.vertical(|ui| {
                    let enabled = base.mods.iter().enumerate()
                        .filter(|(_, m)| m.mod_type().attunement().is_some())
                        .map(|(i,_)| i as u8)
                        .collect();
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
        let idx = self.selected_mod.unwrap() as usize;
        let old_mod = self.selected_mod(&base).unwrap();
        let old_attunement_group = old_mod.mod_type().attunement().unwrap().0;
        let new_attunement_idx = self.material.upgrade().unwrap().attunements.iter().find(|(k, _)| old_attunement_group.kind == *k ).unwrap().1;

        let new_mod = RolledMod {
            mod_id: old_attunement_group[new_attunement_idx].id,
            roll: old_mod.roll,
        };

        stash.remove(self.material.upgrade().unwrap());
        stash.modify(base, |base| base.mods[idx] = new_mod );
    }

    pub fn filter(&self, base: &Item) -> Option<ItemFilter> {
        self.selected_attunement(base).map(|(group, idx)| {
            let attunement = (0..group.kind.len())
                .filter(|i| *i != idx)
                .map(|i| (group.kind, i))
                .collect();

            let excluded = iter::once(base.id).chain(
                self.material.upgrade().map(|item| item.id)
            );

            ItemFilter::of_attunement(attunement, EnumSet::all() - Rating::Favorite, excluded)
        })
    }
}

fn accepts_base(item: &Item) -> bool {
    item.mods.iter().any(|m| m.mod_type().attunement().is_some())
}