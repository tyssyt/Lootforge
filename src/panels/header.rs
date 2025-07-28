use crate::prelude::*;

use crate::widgets::selectable_image::SelectableImage;

use super::settings::SettingsWindow;

pub fn show(ui: &mut Ui, settings: &mut SettingsWindow) -> bool {

    let force_compact_mode = ui.ctx().screen_rect().width() < 1337.;
    if force_compact_mode {
        settings.compact_mode = true;
    }

    ui.horizontal(|ui| {
        #[cfg(target_arch = "wasm32")]
        fullscreen_button(ui);

        let source = include_image!("../../assets/icons/save.png");
        let image = Image::new(source).fit_to_exact_size(vec2(32., 32.));
        let save = ui.add(SelectableImage::new(false, image)).clicked();

        let source = include_image!("../../assets/icons/cog.png");
        let image = Image::new(source).fit_to_exact_size(vec2(32., 32.));
        if ui.add(SelectableImage::new(false, image)).clicked() {
            settings.open();
        }

        if !force_compact_mode {
            ui.checkbox(&mut settings.compact_mode, "Compact View");
        }
        save
    }).inner
}

#[cfg(target_arch = "wasm32")]
fn fullscreen_button(ui: &mut Ui) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(element) = document.document_element() else {
        return;
    };    

    if document.fullscreen_element().is_some() {
        let source = include_image!("../../assets/icons/contract.png");
        let image = Image::new(source).fit_to_exact_size(vec2(32., 32.));
        if ui.add(SelectableImage::new(false, image)).clicked() {
            let _ = document.exit_fullscreen();
        }
    } else {
        let source = include_image!("../../assets/icons/expand.png");
        let image = Image::new(source).fit_to_exact_size(vec2(32., 32.));
        if ui.add(SelectableImage::new(false, image)).clicked() {
            let _ = element.request_fullscreen();
        }
    }
}