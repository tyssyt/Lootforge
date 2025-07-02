use crate::prelude::*;

use crate::widgets::selectable_image::SelectableImage;

use super::settings::SettingsWindow;

pub fn show(ui: &mut Ui, settings: &mut SettingsWindow) -> bool {
    ui.horizontal(|ui| {
        let source = include_image!("../../assets/icons/save.png");
        let image = Image::new(source).fit_to_exact_size(vec2(32., 32.));
        let save = ui.add(SelectableImage::new(false, image)).clicked();

        let source = include_image!("../../assets/icons/cog.png");
        let image = Image::new(source).fit_to_exact_size(vec2(32., 32.));
        if ui.add(SelectableImage::new(false, image)).clicked() {
            settings.open();
        }
        save
    }).inner
}