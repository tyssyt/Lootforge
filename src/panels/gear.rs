use crate::{equipment::{equipment::{EquipChange, FighterEquip}, wardrobe::{ItemSlot, Wardrobe}}, prelude::*, stash::stash::Stash, widgets::item_drop_zone::item_drop_zone};

#[derive(Default)]
pub struct GearPanel {
    open: usize,
}

impl GearPanel {
    pub fn show(&mut self, ui: &mut Ui, wardrobe: &mut Wardrobe, stash: &Stash) {
        ui.heading("Wardrobe");
        ui.horizontal(|ui| {
            ui.style_mut()
                .text_styles
                .get_mut(&TextStyle::Button)
                .unwrap()
                .size = 20.;
            for idx in 0..9 {
                let response = ui.selectable_value(&mut self.open, idx, (idx + 1).to_string());
                if wardrobe.is_equipped(idx) {
                    let corner_radius = ui.style().interact(&response).corner_radius;
                    ui.painter().rect_stroke(
                        response.rect,
                        corner_radius,
                        Stroke::new(2., Color32::LIGHT_GRAY),
                        StrokeKind::Outside,
                    );
                }
            }
        });

        ui.separator();

        if ui
            .add_enabled(!wardrobe.is_equipped(self.open), Button::new("use"))
            .clicked()
        {
            wardrobe.set_equipped(self.open);
        }

        // TODO add unequip logic for things that can be equipped by other chars
        let changes = self.show_fighter_grid(ui, &mut wardrobe.sets[self.open].fighter_equip, stash);
        self.update_changes(changes, wardrobe.is_equipped(self.open));

        // TODO calc some stats and show them
    }

    fn show_fighter_grid(&mut self, ui: &mut Ui, equip: &mut FighterEquip, stash: &Stash) -> Vec<EquipChange> {
        let mut changes = vec![];
        egui::Grid::new("fighter_gear_equip_grid")
            .min_col_width(64.)
            .min_row_height(64.)
            .spacing([5., 5.])
            .show(ui, |ui| {
                ui.label("");
                ui.label("");
                show_item_slot(ui, stash, ItemSlot::Helmet, equip).map(|c| changes.push(c));
                ui.label("");
                ui.end_row();
    
                show_item_slot(ui, stash, ItemSlot::FighterWeapon(0), equip).map(|c| changes.push(c));
                show_item_slot(ui, stash, ItemSlot::FighterWeapon(1), equip).map(|c| changes.push(c));
                show_item_slot(ui, stash, ItemSlot::Armor, equip).map(|c| changes.push(c));
                show_item_slot(ui, stash, ItemSlot::FighterShield, equip).map(|c| changes.push(c));
                ui.end_row();
    
                show_item_slot(ui, stash, ItemSlot::Ring(0), equip).map(|c| changes.push(c));
                show_item_slot(ui, stash, ItemSlot::Ring(1), equip).map(|c| changes.push(c));
                show_item_slot(ui, stash, ItemSlot::Gloves, equip).map(|c| changes.push(c));
                show_item_slot(ui, stash, ItemSlot::Ring(2), equip).map(|c| changes.push(c));
                ui.end_row();
            });
        changes
    }

    fn update_changes(&self, changes: Vec<EquipChange>, is_equipped: bool) {
        for change in changes {
            change.added.users.add_wardrobe(self.open);
            if is_equipped {change.added.users.equipped.set(true)}

            if let Some(item) = change.removed.upgrade() {
                item.users.remove_wardrobe(self.open);
                if is_equipped {item.users.equipped.set(false);}
            }

            if let Some(item) = change.removed2.upgrade() {
                item.users.remove_wardrobe(self.open);
                if is_equipped {item.users.equipped.set(false)}
            }
        }
    }

}

#[must_use]
fn show_item_slot(
    ui: &mut Ui,
    stash: &Stash,
    slot: ItemSlot,
    equip: &mut FighterEquip,
) -> Option<EquipChange> {
    let (item, offhand) = equip.get_item(slot);
    let tint = if offhand {Color32::GRAY} else {Color32::WHITE};

    let (_, dropped_item) = item_drop_zone(ui, stash, |item| slot.accepts(item), |ui| {
        ui.set_min_size(vec2(64., 64.));

        if let Some(item) = item.upgrade() {
            item.show_tinted(ui, tint)
                .on_hover_ui(|ui| item.tooltip(ui));
        } else if let Some(default) = slot.default_type() {
            ui.add(default.image().tint(Color32::DARK_GRAY));
        }
    });

    if let Some(new_item) = dropped_item {
        equip.set_item(new_item, slot)
    } else {
        None
    }
}
