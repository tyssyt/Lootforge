use crate::panels::forge::reroll_random::RerollRandom;
use crate::panels::forge::upgrade::Upgrade;
use crate::prelude::*;

use crate::stash::filters::ItemFilter;
use crate::{
    item::{ItemRef}, stash::stash::Stash, widgets::{
        selectable_image::UiSelectableImage,
    }
};

#[apply(Default)]
pub struct ForgePanel {
    tab: Tab,
    base: ItemRef,

    upgrade: Upgrade,
    reroll_random: RerollRandom,
}

impl ForgePanel {
    pub fn show(&mut self, ui: &mut Ui, stash: &mut Stash) {
        ui.heading("Forge");
        ui.horizontal(|ui| {
            ui.selectable_image(&mut self.tab, Tab::Upgrade, Tab::Upgrade.image());
            ui.selectable_image(&mut self.tab, Tab::RerollRandom, Tab::RerollRandom.image());
            ui.selectable_image(&mut self.tab, Tab::RerollTarget, Tab::RerollTarget.image());
            ui.selectable_image(&mut self.tab, Tab::Attune, Tab::Attune.image());
            ui.selectable_image(&mut self.tab, Tab::Refine, Tab::Refine.image());
            ui.selectable_image(&mut self.tab, Tab::Remove, Tab::Remove.image());
        });

        ui.separator();

        let old_item_id = self.base.upgrade().map_or(0, |i| i.id);
        match self.tab {
            Tab::Upgrade => self.upgrade.show(&mut self.base, ui, stash),
            Tab::RerollRandom => self.reroll_random.show(&mut self.base, ui, stash),
            Tab::RerollTarget => {} // TODO
            Tab::Attune => {}       // TODO
            Tab::Refine => {}       // TODO
            Tab::Remove => {}       // TODO
        }

        let new_item = self.base.upgrade();
        if new_item.as_ref().map_or(0, |i| i.id) != old_item_id {
            self.upgrade = Upgrade::default();
            self.reroll_random = RerollRandom::default();
        }
    }

    pub fn filter(&self) -> Option<ItemFilter> {
        if let Some(base) = self.base.upgrade() {
            match self.tab {
                Tab::Upgrade => Some(self.upgrade.filter(&base)),
                Tab::RerollRandom => Some(self.reroll_random.filter(&base)),
                Tab::RerollTarget => None, // TODO
                Tab::Attune => None, // TODO
                Tab::Refine => None, // TODO
                Tab::Remove => None, // TODO
            }
        } else {
            None
        }
    }
}

#[apply(UnitEnum)]
#[derive(Default)]
enum Tab {
    #[default]
    Upgrade,
    RerollRandom,
    RerollTarget,
    Attune,
    Refine, // or enhance, temper, anneal
    Remove,
}
impl Tab {
    pub fn image(&self) -> Image<'_> {
        use Tab::*;
        let source = match *self {
            Upgrade => include_image!("../../../assets/icons/upgrade.png"),
            RerollRandom => include_image!("../../../assets/icons/cycle.png"),
            RerollTarget => include_image!("../../../assets/icons/card-exchange.png"),
            Attune => include_image!("../../../assets/icons/recycle.png"),
            Refine => include_image!("../../../assets/icons/up-card.png"),
            Remove => include_image!("../../../assets/icons/card-burn.png"),
        };
        Image::new(source).fit_to_exact_size(vec2(32., 32.))
    }
}
