use crate::prelude::*;

pub trait UiSelectableImage {    
    fn selectable_image<'a, Value: PartialEq>(
        &mut self,
        current_value: &mut Value,
        selected_value: Value,
        image: impl Into<Image<'a>>,
    ) -> Response;
}

impl UiSelectableImage for Ui {
    fn selectable_image<'a, Value: PartialEq>(
        &mut self,
        current_value: &mut Value,
        selected_value: Value,
        image: impl Into<Image<'a>>,
    ) -> Response {
        let selected = *current_value == selected_value;

        let mut response = SelectableImage::new(selected, image).ui(self);

        if response.clicked() && !selected {
            *current_value = selected_value;
            response.mark_changed();
        }
        response
    }
}


// Adapted from egui/widgets/selected_label
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct SelectableImage<'a> {
    selected: bool,
    image: Image<'a>,
    alt_text: Option<String>,
}
impl<'a> SelectableImage<'a> {
    pub fn new(selected: bool, image: impl Into<Image<'a>>) -> Self {
        Self { 
            selected,
            image: image.into(),
            alt_text: None,
        }
    }
}
impl<'a> Widget for SelectableImage<'a> {
    fn ui(self, ui: &mut Ui) -> Response {

        let padding = Vec2::splat(ui.spacing().button_padding.x);
        let available_size_for_image = ui.available_size() - 2.0 * padding;
        
        let tlr = self.image.load_for_size(ui.ctx(), available_size_for_image);
        let original_image_size = tlr.as_ref().ok().and_then(|t| t.size());
        let image_size = self
            .image
            .calc_size(available_size_for_image, original_image_size);

        let padded_size = image_size + 2.0 * padding;
        let (rect, response) = ui.allocate_exact_size(padded_size, Sense::click());
        response.widget_info(|| {
            let mut info = WidgetInfo::new(WidgetType::ImageButton);
            info.label = self.alt_text.clone();
            info
        });

        if ui.is_rect_visible(response.rect) {

            let visuals = ui.style().interact_selectable(&response, self.selected);

            if self.selected || response.hovered() || response.highlighted() || response.has_focus() {
                let rect = rect.expand(visuals.expansion);

                ui.painter().rect(
                    rect,
                    visuals.corner_radius,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                    epaint::StrokeKind::Inside,
                );
            }

            let image_rect = ui
                .layout()
                .align_size_within_rect(image_size, rect.shrink2(padding));
            let image_options = self.image.image_options().clone();

            widgets::image::paint_texture_load_result(
                ui,
                &tlr,
                image_rect,
                None,
                &image_options,
                self.alt_text.as_deref(),
            );
        }

        widgets::image::texture_load_result_response(&self.image.source(ui.ctx()), &tlr, response)
    }
}