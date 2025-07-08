use crate::{item::{Item, ItemRef, ItemType}, panels::forge::forge, prelude::*, stash::stash::Stash, widgets::{item_drop_zone::item_drop_zone, selectable_image::SelectableImage}};

pub fn show_item_slot(
    item_ref: &mut ItemRef,
    size: Vec2,
    default: Option<ItemType>,
    ui: &mut Ui,
    stash: &Stash,
    accepts: impl FnOnce(&Item) -> bool,
) -> bool {
    let mut changed = false;
    let (_, dropped_item) = item_drop_zone(ui, stash, accepts, |ui| {
        ui.set_min_size(size);

        if let Some(item) = item_ref.upgrade() {
            let response = item.show_sized(ui, size);
            if response.interact(Sense::click()).clicked_by(PointerButton::Secondary) {
                *item_ref = ItemRef::new();
                changed = true;
            }
            response.on_hover_ui(|ui| item.tooltip(ui));
        } else if let Some(default) = default {
            ui.add(default.image().fit_to_exact_size(size).tint(Color32::DARK_GRAY));
        }
    });

    if let Some(dropped_item) = dropped_item {            
        if item_ref.upgrade().is_none_or(|i| i.id != dropped_item.id) {
            *item_ref = Rc::downgrade(&dropped_item);
            changed = true;
        }
    }

    changed
}

pub fn show_mod_table_single(ui: &mut Ui, item: &Item, selected_mod: &mut Option<u8>, enabled: Vec<u8>) -> bool {
    match show_mod_table(ui, item, |idx| selected_mod.is_some_and(|m| m == idx), enabled) {
        ModTableChange::None => false,
        ModTableChange::Select(true, idx) => { *selected_mod = Some(idx); true },
        ModTableChange::Select(false, _) => { *selected_mod = None; true },
    }
}

pub fn show_mod_table_multi(ui: &mut Ui, item: &Item, selected_mods: &mut Vec<u8>, enabled: Vec<u8>) -> bool {
    match show_mod_table(ui, item, |idx| selected_mods.contains(&(idx)), enabled) {
        ModTableChange::None => false,
        ModTableChange::Select(true, idx) => { selected_mods.push(idx); true },
        ModTableChange::Select(false, idx) => { selected_mods.retain(|i| *i != idx); true },
    }
}

enum ModTableChange {
    None,
    Select(bool, u8),
}
fn show_mod_table(ui: &mut Ui, item: &Item, is_selecetd: impl Fn(u8) -> bool, enabled: Vec<u8>) -> ModTableChange {
    let mut change = ModTableChange::None;
    TableBuilder::new(ui)
        .auto_shrink([false, true])
        .min_scrolled_height(200.)
        .max_scroll_height(250.)
        //.vscroll(false)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(100.).clip(true))
        .column(Column::auto())
        .body(|body| {
            body.rows(20., item.rank() as usize, |mut row| {
                let idx = row.index() as u8;
                let m = &item[idx];
                row.col(|ui| {
                    ui.add_enabled_ui(enabled.contains(&idx), |ui| {
                        if item.rerolled_mod_idx == idx {
                            ui.add(forge::Tab::RerollTarget.image().fit_to_exact_size(vec2(16., 16.)));
                        }
                        m.show_tooltip(ui);
                    });
                });
                row.col(|ui| {
                    ui.add_enabled_ui(enabled.contains(&idx), |ui| {
                        let mut checked = is_selecetd(idx);
                        if ui.checkbox(&mut checked, "").changed() {
                            change = ModTableChange::Select(checked, idx as u8);
                        }
                    });
                });
            });
        });
    change
}

pub fn show_forge_button(enabled: bool, ui: &mut Ui) -> Response {
    // when enabled, I want some shiny border, maybe even animated
    ui.vertical_centered(|ui| {
        ui.style_mut().visuals.selection.bg_fill = Color32::TRANSPARENT; //Color32::from_rgb(53, 92, 125);

        let color = if enabled {
            Color32::DARK_RED
        } else {
            Color32::GRAY
        };
        let source = include_image!("../../../assets/icons/anvil-impact.png");
        let image = Image::new(source)
            .fit_to_exact_size(vec2(64., 64.))
            .tint(color);
        // TODO wtf it is squished??? maybe the img is not but the button is not square :(
        ui.add_enabled(enabled, SelectableImage::new(enabled, image))
    })
    .inner
}