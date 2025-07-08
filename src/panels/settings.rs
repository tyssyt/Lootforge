use crate::{prelude::*, widgets::selectable_image::SelectableImage};

#[apply(Default)]
pub struct SettingsWindow {
    open: bool,
    delete_confirm_open: bool,
}
impl SettingsWindow {
    pub fn open(&mut self) {
        self.open = true;
    }
    
    // TODO show game time
    pub fn show(&mut self, ctx: &Context) -> (bool, bool) {
        let cheats_opened = Window::new("Settings")
            .open(&mut self.open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let source = include_image!("../../assets/icons/trash-can.png");
                    let image = Image::new(source).fit_to_exact_size(vec2(32., 32.));
                    if ui.add(SelectableImage::new(false, image)).clicked() {
                        self.delete_confirm_open = true;
                    }
                    ui.label("delete save");
                });

                if cfg!(debug_assertions) {
                    ui.separator();
                    ui.button("Cheats").clicked()
                } else {
                    false
                }
        }).map_or(false, |a| a.inner.unwrap_or(false));

        let delete_save = self.confirm_delete(ctx);
        (delete_save, cheats_opened)
    }

    fn confirm_delete(&mut self, ctx: &Context) -> bool {
        Window::new("Confirm Delete")
            .open(&mut self.delete_confirm_open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("This will delete your save file. There is no way to recover it. Are you sure?");
                ui.vertical_centered(|ui| {
                    ui.button("Start again").clicked()
                }).inner
        }).map_or(false, |a| a.inner.unwrap_or(false))
    }
}