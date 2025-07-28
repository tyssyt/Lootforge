use egui_double_slider::DoubleSlider;

use crate::item::item_type::ItemType;
use crate::item::tags::Rating;
use crate::mods::{roll_tables, ModType};
use crate::prelude::*;
use crate::stash::filters::ItemFilter;
use crate::stash::stash::Stash;
use crate::stash::order::Order;
use crate::widgets::selectable_image::SelectableImage;

#[apply(Default)]
pub struct LootPanel {
    order: Order,
    filter: ItemFilter,
    search_text: String,
    #[default(roll_tables::ALL_MODS.values().map(|m| *m).collect())]
    shown_mods: Vec<&'static ModType>,
}
impl LootPanel {
    pub fn show(&mut self, ui: &mut Ui, stash: &mut Stash, filter_override: Option<&ItemFilter>) {
        ui.heading("Loot");
        self.show_filters(stash.max_rank(), ui);
        ui.separator();
        self.show_order(ui);
        ui.separator();
        self.show_items(ui, stash, filter_override);
    }

    fn show_filters(&mut self, max_rank: u8, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Filters:"));
            self.show_type_filter(ui);
            ui.add_space(3.0);
            self.show_rating_filter(ui);
            ui.add_space(3.0);
            self.show_rank_filter(max_rank, ui);
            ui.add_space(3.0);
            self.show_mod_filter(ui);
        });

        if ui.button("Reset All").clicked() {
            self.filter.clear();
        }
    }

    fn show_type_filter(&mut self, ui: &mut Ui) {
        ui.menu_button("Type", |ui| {
            ui.set_max_width(3.0 * 85.0);
            ui.horizontal_wrapped(|ui|{
                for item_type in ItemType::iter() {
                    if ui.add(SelectableImage::new(self.filter.has_type(item_type), item_type.image())).clicked() {
                        self.filter.toggle_type(item_type);
                    }
                }
            });
        });
    }

    fn show_rating_filter(&mut self, ui: &mut Ui) {
        ui.menu_button("Rating", |ui| {            
            for rating in Rating::iter() {
                if rating_button(ui, rating, self.filter.has_rating(rating)) {
                    self.filter.toggle_rating(rating);
                }
            }
        });
    }

    fn show_rank_filter(&mut self, max_rank: u8, ui: &mut Ui) {
        ui.menu_button("Rank", |ui| {
            let (mut lower, mut upper) = self.filter.ranks().into_inner();

            ui.vertical_centered(|ui| {
                ui.label(format!("Rank: {}-{}", lower, upper));
            });
            ui.add(
                DoubleSlider::new(&mut lower, &mut upper, 1..=max_rank)
                .width(300.0)
                .separation_distance(0)
            );

            self.filter.set_ranks(lower..=upper);
        });
    }

    fn show_mod_filter(&mut self,ui: &mut Ui) {
        ui.menu_button("Mods", |ui| {
            if ui.add(TextEdit::singleline(&mut self.search_text).hint_text("search")).changed() {
                if self.search_text.is_empty() {
                    self.shown_mods = roll_tables::ALL_MODS.values().map(|m| *m).collect();
                } else {
                    // TODO also check the tooltip
                    self.shown_mods = roll_tables::ALL_MODS.values()
                        .map(|m| *m)
                        .filter(|m| m.prefix_name.contains(&self.search_text))
                        .collect()
                }
            }
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                for item_mod in &self.shown_mods {
                    let mut checked = self.filter.has_mod(item_mod.id, 1);
                    let response = ui.checkbox(&mut checked, item_mod.prefix_name);

                    if response.changed() {
                        if checked {
                            self.filter.add_mod(item_mod.id, 1);
                        } else {
                            self.filter.remove_mod(item_mod.id);
                        }
                    }

                    response.on_hover_ui(|ui| {
                        // TODO make a tooltip variant without roll          
                        (item_mod.show_tooltip)(item_mod, ui, *item_mod.roll_range.end());
                    });
                }
            });
        });
    }

    fn show_order(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Order by: ");            
            ComboBox::from_id_salt("Loot Sorting Order")
                .selected_text(format!("{}", self.order))
                .show_ui(ui, |ui| {
                    for order in Order::iter() {
                        ui.selectable_value(&mut self.order, order, order.to_string());
                    }
            });
        });
    }

    fn show_items(&mut self, ui: &mut Ui, stash: &mut Stash, filter_override: Option<&ItemFilter>) {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for item in stash.filtered_items(filter_override.unwrap_or(&self.filter), self.order).iter() {
                    // without this surrounding ui the elements will not wrap around to a new line
                    // TODO with every update of egui, check if this is still necessary
                    ui.allocate_ui(vec2(64., 64.), |ui| {
                        let item_id = Id::new(("dnd_item", item.id));
                        let response = ui.dnd_drag_source(item_id, item.id, |ui| {
                            item.show(ui);
                        }).response;

                        response.context_menu(|ui| {
                            ui.label("Mark item as");
                            ui.separator();

                            for rating in Rating::iter() {
                                if rating_button(ui, rating, false) {
                                    item.tags.set_rating(rating, stash);
                                    ui.close_menu();
                                }
                            }
                        });

                        response.on_hover_ui(|ui| item.tooltip(ui));
                    });
                }
            });
        });
    }
}

fn rating_button(ui: &mut Ui, rating: Rating, selected: bool) -> bool {
    let text: &'static str = rating.into();
    if let Some(image) = rating.image() {
        ui.add(Button::image_and_text(image, text).selected(selected)).clicked()
    } else {
        ui.add(Button::new(text).selected(selected)).clicked()
    }

}