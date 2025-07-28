use crate::equipment::wardrobe::Wardrobe;
#[cfg(debug_assertions)]
use crate::panels::cheats::CheatsWindow;
use crate::panels::forge::forge::ForgePanel;
use crate::panels::header;
use crate::panels::settings::SettingsWindow;
use crate::storage::storage_manager::{LoadingState, StorageManager};
use crate::prelude::*;
use crate::{
    dungeon::dungeon_data::DungeonData,
    panels::{
        dungeon::dungeon::DungeonPanel, gear::GearPanel, loot::LootPanel,
        rewards::RewardsWindow,
    },
    stash::stash::Stash,
    timekeeper::Timekeeper,
};

#[derive(Debug, SmartDefault)]
pub struct LootforgeApp {
    pub left_panel: LeftPanel,

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
    #[cfg(debug_assertions)]
    pub cheats: CheatsWindow,
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
        let just_finished_loading = match StorageManager::loading(self) {
            LoadingState::None => false,
            LoadingState::Done(_) => true,
            LoadingState::Loading(_) => {
                ctx.request_repaint();
                // TODO drawn loading symbol
                return;
            },
        };

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
        self.dungeon_panel.tick(&self.dungeon, &frame_info, just_finished_loading);


        let save = TopBottomPanel::top("Header Bar")
            .min_height(32.)
            .resizable(false)
            .show(ctx, |ui| {
                header::show(ui, &mut self.settings)
            }).inner;

        if !self.settings.compact_mode && self.left_panel == LeftPanel::Dungeon {
            self.left_panel = Default::default();
        }

        let mut left_panel = SidePanel::left("forge & gear").resizable(false);
        if self.left_panel != LeftPanel::Dungeon {
            left_panel = left_panel.exact_width(450.);
        }
        left_panel.show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.left_panel, LeftPanel::Forge, RichText::new("Forge").heading());
                ui.selectable_value(&mut self.left_panel, LeftPanel::Wardrobe, RichText::new("Wardrobe").heading());
                if self.settings.compact_mode {
                    ui.selectable_value(&mut self.left_panel, LeftPanel::Dungeon, RichText::new("Dungeon").heading());
                }
            });
            ui.separator();
            match self.left_panel {
                LeftPanel::Forge    => self.forge.show(ui, &mut self.stash),
                LeftPanel::Wardrobe => self.gear_panel.show(ui, &mut self.wardrobe, &self.stash),
                LeftPanel::Dungeon  => self.dungeon_panel.show(ui, &mut self.dungeon, &self.wardrobe, &mut self.rewards, &frame_info),
            }
        });

        if !self.settings.compact_mode {
            SidePanel::right("dungeon")
                .resizable(false)
                .show(ctx, |ui| {
                    self.dungeon_panel.show(ui, &mut self.dungeon, &self.wardrobe, &mut self.rewards, &frame_info);
            });
        }

        CentralPanel::default().show(ctx, |ui| {
            let filter_override = if self.left_panel == LeftPanel::Forge { self.forge.filter() } else { None };
            self.loot_panel.show(ui, &mut self.stash, filter_override);
        });

        self.rewards.show(ctx, &mut self.dungeon, &mut self.stash);
        let (delete_save, cheats_opened) = self.settings.show(ctx);

        #[cfg(debug_assertions)]
        if cheats_opened {
            self.cheats.open = true;
        }
        
        #[cfg(debug_assertions)]
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

#[apply(UnitEnum)]
#[derive(Default)]
pub enum LeftPanel {
    #[default]
    Forge,
    Wardrobe,
    Dungeon,
}