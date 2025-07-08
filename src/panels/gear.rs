use crate::{equipment::{equipment::{Equip, EquipChange, FighterEquip}, wardrobe::{ItemSlot, Wardrobe}}, explorer::Explorer, item::ItemRef, prelude::*, stash::stash::Stash, widgets::item_drop_zone::item_drop_zone};

#[apply(Default)]
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

        if ui.add_enabled(!wardrobe.is_equipped(self.open), Button::new("use")).clicked() {
            wardrobe.set_equipped(self.open);
        }

        // TODO add unequip logic for things that can be equipped by other chars
        let changes = self.show_fighter_grid(ui, &mut wardrobe.sets[self.open].fighter_equip, stash);
        self.update_changes(changes, wardrobe.is_equipped(self.open));

        // TODO calc some stats and show them
    }

    fn show_fighter_grid(&mut self, ui: &mut Ui, equip: &mut FighterEquip, stash: &Stash) -> Vec<EquipChange> {
        let mut changes = vec![];
        let responses = egui::Grid::new("fighter_gear_equip_grid")
            .min_col_width(64.)
            .min_row_height(64.)
            .spacing([5., 5.])
            .show(ui, |ui| {
                ui.label("");
                ui.label("");
                show_item_slot_fighter(ui, stash, ItemSlot::Helmet, equip, &mut changes);
                ui.label("");
                ui.end_row();
    
                let w1 = show_item_slot_fighter(ui, stash, ItemSlot::Weapon(0), equip, &mut changes);
                let w2 = show_item_slot_fighter(ui, stash, ItemSlot::Weapon(1), equip, &mut changes);
                show_item_slot_fighter(ui, stash, ItemSlot::Armor, equip, &mut changes);
                let s = show_item_slot_fighter(ui, stash, ItemSlot::FighterShield, equip, &mut changes);
                ui.end_row();
    
                let r1 = show_item_slot_fighter(ui, stash, ItemSlot::Ring(0), equip, &mut changes);
                let r2 = show_item_slot_fighter(ui, stash, ItemSlot::Ring(1), equip, &mut changes);
                show_item_slot_fighter(ui, stash, ItemSlot::Gloves, equip, &mut changes);
                let r3 = show_item_slot_fighter(ui, stash, ItemSlot::Ring(2), equip, &mut changes);
                ui.end_row();
                [(w1, r1), (w2, r2),  (s, r3)]
            }).inner;

            connect_slots(ui, equip.weapons[0].clone(), equip.common.rings[0].clone(), &responses[0].0, &responses[0].1);
            connect_slots(ui, equip.weapons[1].clone(), equip.common.rings[1].clone(), &responses[1].0, &responses[1].1);
            connect_slots(ui, equip.shield.clone(), equip.common.rings[2].clone(), &responses[2].0, &responses[2].1);


        changes
    }

    fn update_changes(&self, changes: Vec<EquipChange>, is_equipped: bool) {
        for change in changes {
            if let Some(item) = change.added.upgrade() {
                item.users.add_wardrobe(self.open);
                if is_equipped {item.users.equipped.set(true)}
            }

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

// TODO don't need copies for other explorer, this is generelizable
fn show_item_slot_fighter(
    ui: &mut Ui,
    stash: &Stash,
    slot: ItemSlot,
    equip: &mut FighterEquip,
    changes: &mut Vec<EquipChange>,
) -> Response {
    let (item, offhand) = equip.get_item(slot);
    let tint = if offhand {Color32::GRAY} else {Color32::WHITE};

    let (response, dropped_item) = item_drop_zone(ui, stash, |item| slot.accepts(Explorer::Fighter, item), |ui| {
        ui.set_min_size(vec2(64., 64.));

        if let Some(item) = item.upgrade() {
            let response = item.show_tinted(ui, tint);            
            if response.interact(Sense::click()).clicked_by(PointerButton::Secondary) {
                equip.set_item(ItemRef::new(), slot).map(|c| changes.push(c));
            }
            response.on_hover_ui(|ui| item.tooltip(ui));
        } else if let Some(default) = slot.default_type(Explorer::Fighter) {
            ui.add(default.image().tint(Color32::DARK_GRAY));
        }
    });

    if let Some(new_item) = dropped_item {
        equip.set_item(Rc::downgrade(&new_item), slot).map(|c| changes.push(c));
    }

    response.response
}

fn connect_slots(ui: &mut Ui, item1: ItemRef, item2: ItemRef, response1: &Response, response2: &Response) {
    if item1.upgrade().is_none() || item2.upgrade().is_none() {
        return;
    }

    ui.painter().line(vec![response1.rect.center_bottom() - vec2(0., 7.), response2.rect.center_top() + vec2(0., 7.)], (2.0, Color32::WHITE));
}