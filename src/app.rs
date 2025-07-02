use crate::equipment::wardrobe::Wardrobe;
use crate::panels::cheats::CheatsWindow;
use crate::panels::forge::forge::ForgePanel;
use crate::panels::header;
use crate::panels::settings::SettingsWindow;
use crate::storage::storage_manager::StorageManager;
use crate::prelude::*;
use crate::{
    dungeon::dungeon::DungeonData,
    panels::{
        dungeon::DungeonPanel, gear::GearPanel, loot::LootPanel,
        rewards::RewardsWindow,
    },
    stash::stash::Stash,
    timekeeper::Timekeeper,
};

#[derive(Default)]
pub struct LootforgeApp {
    pub storage_manager: StorageManager,
    pub timekeeper: Timekeeper,
    pub stash: Stash,
    pub dungeon: DungeonData,
    pub forge: ForgePanel,
    pub wardrobe: Wardrobe,
    pub dungeon_panel: DungeonPanel,
    pub gear_panel: GearPanel,
    pub loot_panel: LootPanel,
    pub rewards: RewardsWindow,
    pub settings: SettingsWindow,
    pub cheats: CheatsWindow, // TODO disable on release builds
}

impl LootforgeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        use FontFamily::Proportional;
        use TextStyle::*;

        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_theme(ThemePreference::Dark);
        cc.egui_ctx.options_mut(|options| {
            options.reduce_texture_memory = true;
        });
        cc.egui_ctx.style_mut(|style| {
            style.visuals = Visuals::dark();
            style.spacing.scroll.floating = false;
            style.text_styles = [
                (Heading, FontId::new(30.0, Proportional)),
                //(heading2(), FontId::new(25.0, Proportional)),
                //(heading3(), FontId::new(23.0, Proportional)),
                (Body, FontId::new(14.0, Proportional)),
                (Monospace, FontId::new(14.0, Proportional)),
                (Button, FontId::new(16.0, Proportional)),
                (Small, FontId::new(10.0, Proportional)),
            ]
            .into()
        });

        let mut app = LootforgeApp::default();
        app.storage_manager.load_quicksaves_after_init = true;
        app
    }
}

impl eframe::App for LootforgeApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if StorageManager::loading(self) {
            ctx.request_repaint();
            // TODO drawn loading symbol
            return;
        }

        let mut frame_info = self.timekeeper.update(ctx);
        if let Some(target_frames) = frame_info.catch_up {
            // TODO visualize catch up
            let start = web_time::Instant::now();
            let mut frames = 0;
            let mut dungeon_tick = None;
            while start.elapsed().as_millis() < 100 && target_frames - frames > 100 {
                for _ in 0..100 {
                    dungeon_tick = self.dungeon.tick(self.wardrobe.equipped())
                }
                frames += 100;
            }
            if target_frames - frames < 100 {
                for _ in 0..target_frames - frames {
                    dungeon_tick = self.dungeon.tick(self.wardrobe.equipped())
                }
                frames = target_frames;
            }
            frame_info.dungeon_tick = dungeon_tick;
            self.timekeeper.report_frames(frames);
        } else if frame_info.tick {
            let dungeon_tick = self.dungeon.tick(self.wardrobe.equipped());
            frame_info.dungeon_tick = dungeon_tick;
            self.timekeeper.report_frames(1);
        }

        if ctx.available_rect().width() < 1280. || ctx.available_rect().height() < 720. {
            // TODO use a scene to enable zoom? or look into the source code of scene how it does it!
            // afaics this won't work herem but needs to be within a ui, so the panel approach below won't work
            // but maybe with groups or somthing

            // weirdly, zoom with ctrl + +/- works in browser and native, but not ctrl + scrollweel
            // zooming in browser requires me setting something in aceessebility settings
            // and even then i can only zoom in, and not zoom at all on the desktop version toggle
        }

        let save = TopBottomPanel::top("Header Bar")
            .min_height(32.)
            .resizable(false)
            .show(ctx, |ui| {
                header::show(ui, &mut self.settings)
            }).inner;

        SidePanel::left("forge & gear")
            .min_width(420.)
            .resizable(false)
            .show(ctx, |ui| {
                self.forge.show(ui, &mut self.stash);
                ui.separator();
                self.gear_panel.show(ui, &mut self.wardrobe, &self.stash);
            });
        SidePanel::left("workbench & dungeon")
            .resizable(false)
            .show(ctx, |ui| {
                self.dungeon_panel.show(
                    ui,
                    &mut self.dungeon,
                    &self.wardrobe,
                    &mut self.rewards,
                    frame_info,
                );
            });

        CentralPanel::default().show(ctx, |ui| {
            self.loot_panel.show(ui, &mut self.stash, self.forge.filter());
        });

        self.rewards.show(ctx, &mut self.dungeon, &mut self.stash);
        let (delete_save, cheats_opened) = self.settings.show(ctx);

        if cheats_opened {
            self.cheats.open = true;
        }
        self.cheats.show(ctx, &mut self.stash, &mut self.dungeon, &mut self.timekeeper);

        if delete_save {
            *self = Self::default();
            self.stash.give_starting_items();
        }

        if save {
            StorageManager::def_save(self);
        } else {
            StorageManager::maybe_save(self);
        }
    }
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        info!("exiting - saving now");
        StorageManager::def_save(self);
    }
}
