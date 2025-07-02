use crate::{item::{Item, ItemRef, ItemType}, prelude::*, stash::stash::Stash, widgets::{item_drop_zone::item_drop_zone, selectable_image::SelectableImage}};

pub fn show_item_slot(
    item: &mut ItemRef,
    size: Vec2,
    default: Option<ItemType>,
    ui: &mut Ui,
    stash: &Stash,
    accepts: impl FnOnce(&Item) -> bool,
) -> bool {
    let (response, dropped_item) = item_drop_zone(ui, stash, accepts, |ui| {
        ui.set_min_size(size);

        if let Some(item) = item.upgrade() {
            item.show_sized(ui, size).on_hover_ui(|ui| item.tooltip(ui));
        } else if let Some(default) = default {
            ui.add(default.image().fit_to_exact_size(size).tint(Color32::DARK_GRAY));
        }
    });

    if item.upgrade().is_some() && response.response.interact(Sense::click()).clicked_by(PointerButton::Secondary) {
        *item = ItemRef::new();
        return true;
    }

    if let Some(dropped_item) = dropped_item {            
        if item.upgrade().is_none_or(|i| i.id != dropped_item.id) {
            *item = Rc::downgrade(&dropped_item);
            return true;
        }
    }

    false
}

pub fn show_mod_table(ui: &mut Ui, item: &Item, selected_mods: &mut Vec<usize>) {
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
                let idx = row.index();
                let m = &item.mods[idx];
                row.col(|ui| {
                    m.show_tooltip(ui);
                });
                row.col(|ui| {
                    let mut checked = selected_mods.contains(&idx);
                    if ui.checkbox(&mut checked, "").changed() {
                        if checked {
                            selected_mods.push(idx);
                        } else {
                            selected_mods.retain(|i| *i != idx);
                        }
                    }
                });
            });
        });
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