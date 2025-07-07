use std::iter;

use crate::mods::roll_tables::ALL_MODS;
use crate::prelude::*;
use super::common::*;

use crate::item::{Item, ItemRef};
use crate::stash::filters::ItemFilter;
use crate::stash::stash::Stash;

#[apply(Default)]
pub struct Upgrade {
    selected_mods: Vec<usize>,
    materials: [ItemRef; 9],
}

impl Upgrade {    
    fn selected_mod_counts(&self, base: &Item) -> impl Iterator<Item = (u16, u8)> {
        self.selected_mods.iter()
            .map(|i| base.mods[*i].mod_id)
            .counts().into_iter()
            .map(|(m, c)| (m, c as u8))
    }

    pub fn show(&mut self, base_ref: &mut ItemRef, ui: &mut Ui, stash: &mut Stash) {
        // more styling and stuff
        ui.label("Upgrade 10 Items of the same rank to an item a rank higher");
        ui.label("If a modifier is present on all 10 items, it will be transferred to the upgraded item");

        ui.add_space(8.);

        if let Some(base) = base_ref.upgrade() {
            // toggle button that filters stash for right items

            ui.horizontal_top(|ui| {
                show_item_slot(base_ref, vec2(64., 64.), None, ui, stash, accepts_base);
                ui.vertical(|ui| show_mod_table(ui, &base, &mut self.selected_mods));
            });

            self.remove_invalid_materials(&base);

            ui.add_space(8.);

            ui.horizontal_wrapped(|ui| {
                let invalid_mats: Vec<_> = self.materials.iter()
                    .filter_map(|mat| mat.upgrade())
                    .chain(once(base.clone()))
                    .collect();

                for material in &mut self.materials {
                    show_item_slot(material, vec2(32., 32.), Some(base.item_type), ui, stash, |mat| accepts_mat(mat, &base, &invalid_mats));
                }
            });

            ui.add_space(8.);

            let enabled = self.materials.iter().all(|mat| mat.upgrade().is_some());
            if show_forge_button(enabled, ui).clicked() {
                self.forge(base, stash);
            }
        } else {
            ui.vertical_centered(|ui| {
                show_item_slot(base_ref, vec2(64., 64.), None, ui, stash, accepts_base);
            });
        }
    }

    fn remove_invalid_materials(&mut self, base: &Item) {
        // TODO it's inefficient to run selected_mod_counts for every material
        for i in 0..self.materials.len() {
            if self.materials[i].upgrade().is_some_and(|mat| !mat.has_all_mods(self.selected_mod_counts(base))) {
                self.materials[i] = ItemRef::new();
            }
        }
    }

    fn forge(&mut self, base: Rc<Item>, stash: &mut Stash) {
        let mut rng = rand::rng();
        let protected_mods = self.protected_mods(&base).into_iter()
            .flat_map(|(m, c)| iter::repeat_n(m, c as usize))
            .map(|m| ALL_MODS[&m].roll(&mut rng))
            .collect();
        let new_item = Item::random_of_type(&mut rng, base.item_type, base.rank() + 1, protected_mods);

        self.materials.iter().for_each(|mat| stash.remove(mat.upgrade().unwrap()));
        stash.remove(base);
        stash.add(new_item);
    }

    fn protected_mods(&self, base: &Item) -> Vec<(u16, u8)>  {
        let materials: Vec<_> = self.materials.iter()
            .map(|mat| mat.upgrade().unwrap())
            .collect();
        base.mods.iter()
            .map(|m| m.mod_id)
            .counts().into_iter()
            .map(|(m, c)| (m, c as u8))
            .map(|(m, c)| (m, c.at_most(material_mod_count(m, &materials))))
            .filter(|(_, c)| *c > 0)
            .collect()
    }

    pub fn filter(&self, base: &Item) -> ItemFilter {
        let excluded = self.materials.iter()
            .filter_map(|m| m.upgrade())
            .map(|m| m.id)
            .chain(iter::once(base.id));

        ItemFilter::new(base.item_type, base.rank(), self.selected_mod_counts(base), excluded)
    }
}

fn accepts_base(_item: &Item) -> bool {
    true
}
fn accepts_mat(mat: &Item, base: &Item, invalid_mats: &Vec<Rc<Item>>) -> bool {
    mat.item_type == base.item_type
        && mat.rank() == base.rank()
        && invalid_mats.iter().all(|invalid| mat != invalid.as_ref())
}

fn material_mod_count(mod_id: u16, materials: &Vec<Rc<Item>>) -> u8 {
    materials.iter()
        .map(|mat| mat.has_mod(mod_id))
        .min().unwrap()
}