use crate::panels::forge::attune::Attune;
use crate::panels::forge::reroll_random::RerollRandom;
use crate::panels::forge::reroll_target::RerollTarget;
use crate::panels::forge::upgrade::Upgrade;
use crate::prelude::*;

use crate::stash::filters::ItemFilter;
use crate::widgets::selectable_image::SelectableImage;
use crate::{
    item::item::ItemRef, stash::stash::Stash, widgets::{
        selectable_image::UiSelectableImage,
    }
};

#[apply(Default)]
pub struct ForgePanel {
    tab: Tab,
    base: ItemRef,
    cached_filter: Option<Option<ItemFilter>>,

    upgrade: Upgrade,
    reroll_random: RerollRandom,
    reroll_target: RerollTarget,
    attune: Attune,
}

impl ForgePanel {
    pub fn show(&mut self, ui: &mut Ui, stash: &mut Stash) {
        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= ui.selectable_image(&mut self.tab, Tab::Upgrade, Tab::Upgrade.image()).changed();
            changed |= ui.selectable_image(&mut self.tab, Tab::RerollRandom, Tab::RerollRandom.image()).changed();
            changed |= ui.selectable_image(&mut self.tab, Tab::RerollTarget, Tab::RerollTarget.image()).changed();
            changed |= ui.selectable_image(&mut self.tab, Tab::Attune, Tab::Attune.image()).changed();
            changed |= ui.add_enabled(false, SelectableImage::new(false, Tab::Refine.image())).changed();
            // changed |= ui.selectable_image(&mut self.tab, Tab::Refine, Tab::Refine.image()).changed();
            changed |= ui.add_enabled(false, SelectableImage::new(false, Tab::Remove.image())).changed();
            // changed |= ui.selectable_image(&mut self.tab, Tab::Remove, Tab::Remove.image()).changed();
        });

        ui.separator();

        let old_item_id = self.base.upgrade().map_or(0, |i| i.id);
        changed |= match self.tab {
            Tab::Upgrade      => self.upgrade.show(&mut self.base, ui, stash),
            Tab::RerollRandom => self.reroll_random.show(&mut self.base, ui, stash),
            Tab::RerollTarget => self.reroll_target.show(&mut self.base, ui, stash),
            Tab::Attune       => self.attune.show(&mut self.base, ui, stash),
            Tab::Refine => false,       // TODO
            Tab::Remove => false,       // TODO
        };

        if changed {
            self.cached_filter = None;
        }

        let new_item = self.base.upgrade();
        if new_item.as_ref().map_or(0, |i| i.id) != old_item_id {
            self.upgrade = Upgrade::default();
            self.reroll_random = RerollRandom::default();
            self.reroll_target = RerollTarget::default();
            self.attune = Attune::default();
        }
    }

    pub fn filter(&mut self) -> Option<&ItemFilter> {
        if self.cached_filter.is_none() {
            self.cached_filter = Some(self.create_filter());
        }
        self.cached_filter.as_ref().unwrap().as_ref()
    }

    fn create_filter(&self) -> Option<ItemFilter> {
        if let Some(base) = self.base.upgrade() {
            match self.tab {
                Tab::Upgrade => Some(self.upgrade.filter(&base)),
                Tab::RerollRandom => Some(self.reroll_random.filter(&base)),
                Tab::RerollTarget => Some(self.reroll_target.filter(&base)),
                Tab::Attune => self.attune.filter(&base),
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
pub enum Tab {
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
