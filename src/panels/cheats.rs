use std::time::Duration;

use crate::{dungeon::{dungeon_data::DungeonData, reward::RewardChest}, item::{Item, ItemType}, prelude::*, stash::stash::Stash, timekeeper::Timekeeper};

#[apply(Default)]
pub struct CheatsWindow {
    pub open: bool,
    item_adder: ItemAdder,
    chest_adder: ChestAdder,
    time_skipper: TimeSkipper,
}
impl CheatsWindow {
    pub fn show(&mut self, ctx: &Context, stash: &mut Stash, dungeon: &mut DungeonData, timekeeper: &mut Timekeeper) {     
        Window::new("Cheats")
            .open(&mut self.open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                self.item_adder.show(stash, ui);
                ui.separator();
                self.chest_adder.show(dungeon, ui);
                ui.separator();
                self.time_skipper.show(timekeeper, ui);
        });
    }
}

#[apply(Default)]
struct ItemAdder {
    #[default(ItemType::Axe)]
    item_type: ItemType,
    #[default(1)]
    rank: u8,
}
impl ItemAdder {
    fn show(&mut self, stash: &mut Stash, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ComboBox::from_id_salt("Item Type Selector")
                .selected_text(format!("{:?}", self.item_type))
                .show_ui(ui, |ui| {
                    for i in ItemType::iter() {
                        ui.selectable_value(&mut self.item_type, i, format!("{:?}", i));
                    }
                });

            ui.add(DragValue::new(&mut self.rank).range(1..=10));

            if ui.button("Add Item").clicked() {
                stash.add(Item::random(&mut rand::rng(), self.item_type, self.rank));
            }
        });
    }
}

#[apply(Default)]
struct ChestAdder {
    depth: u16,
}
impl ChestAdder {
    fn show(&mut self, dungeon: &mut DungeonData, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.add(DragValue::new(&mut self.depth).range(1..=100));
            if ui.button("Add Chest").clicked() {
                dungeon.rewards.entry(self.depth).or_default().push(RewardChest::from(&mut rand::rng(), self.depth));
            }
        });
    }
}

#[apply(Default)]
struct TimeSkipper {
}
impl TimeSkipper {
    fn show(&mut self, timekeeper: &mut Timekeeper,  ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Skip 10m").clicked() {
                timekeeper.last_save_sim -= Duration::from_secs(10*60);
            }
            if ui.button("Skip 1h").clicked() {
                timekeeper.last_save_sim -= Duration::from_secs(60*60);
            }
            if ui.button("Skip 24h").clicked() {
                timekeeper.last_save_sim -= Duration::from_secs(24*60*60);
            }
        });
    }
}