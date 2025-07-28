use super::{item_type::ItemType, tags::ItemTags};

use crate::item::tags::Rating;
use crate::mods::attune::Attunement;
use crate::panels::forge::forge;
use crate::prelude::*;

use crate::{
    combat::skill::{targeting::Targeting},
    mods::*,
    widgets::text_in_rect::text_in_rect,
};


#[derive(Debug)]
pub struct Item {
    pub id: usize,
    pub item_type: ItemType,
    // TODO most items don't even have targeting, but I do need something to store inherit stuff, prolly in itemtype data
    pub targeting: Option<Targeting>,
    pub mods: Vec<RolledMod>,
    pub rerolled_mod_idx: u8,
    pub tags: ItemTags, // TODO we can add a Box here for pointer and then do some mem optimization, because most items have similar tags
    pub attunements: Vec<Attunement>,
}

pub type ItemRef = Weak<Item>;

impl Item {
    pub fn new(item_type: ItemType, mods: Vec<RolledMod>, targeting: Option<Targeting>) -> Self {
        let mut item = Self {
            id: 0,
            item_type,
            targeting,
            mods,
            rerolled_mod_idx: u8::MAX,
            tags: ItemTags::default(),
            attunements: Vec::new(),
        };
        item.recompute_attunements();
        item
    }
    pub fn random(rng: &mut impl Rng, item_type: ItemType, rank: u8) -> Self {
        Self::random_with_mods(rng, item_type, rank, Vec::new())
    }
    pub fn random_with_mods(rng: &mut impl Rng, item_type: ItemType, rank: u8, forced_mods: Vec<RolledMod>) -> Self {
        let mut mods = Vec::with_capacity(rank as usize);

        mods.extend(forced_mods);

        for _ in mods.len()..rank as usize {
            mods.push(item_type.roll_mod(rng, &mods));
        }

        let targeting = if item_type.has_targeting() {
            Some(Targeting::roll_ring(rng))
        } else {
            None
        };

        Self::new(item_type, mods, targeting)
    }

    pub fn recompute_attunements(&mut self) {
        self.attunements = self.mods.iter()
            .filter_map(|m| m.mod_type().attunement())
            .map(|(group, idx )| (group.kind, idx))
            .into_group_map()
            .iter()
            .filter(|(_, idx)| idx.iter().all_equal())
            .map(|(kind, idx)| (*kind, idx[0]))
            .collect();
    }

    pub fn rank(&self) -> u8 {
        self.mods.len().try_into().expect("item has more then 255 mods, WTF")
    }

    pub fn mod_count(&self, mod_id: u16) -> u8 {
        self.mods.iter().filter(|m| m.mod_id == mod_id).count() as u8
    }

    pub fn has_mod(&self, mod_id: u16) -> bool {
        self.mod_count(mod_id) > 0
    }

    pub fn has_all_mods<'a>(&self, mut mods: impl Iterator<Item = (u16, u8)>) -> bool {
        mods.all(|(wanted, count)| self.mod_count(wanted) >= count)
    }

    pub fn rerolled_mod_idx(&self) ->  Option<u8> {
        if self.rerolled_mod_idx == u8::MAX {
            None
        } else {
            Some(self.rerolled_mod_idx)
        }
    }

    pub fn show(&self, ui: &mut Ui) -> Response {
        self.show_tinted_sized(ui, Color32::WHITE, ItemType::SIZE)
    }

    pub fn show_sized(&self, ui: &mut Ui, size: Vec2)  -> Response {
        self.show_tinted_sized(ui, Color32::WHITE, size)
    }

    pub fn show_tinted(&self, ui: &mut Ui, tint: Color32) -> Response {
        self.show_tinted_sized(ui, tint, ItemType::SIZE)
    }

    pub fn show_tinted_sized(&self, ui: &mut Ui, tint: Color32, size: Vec2) -> Response {
        let response = ui.add(self.item_type.image().tint(tint).fit_to_exact_size(size));

        text_in_rect(ui, RichText::new(self.rank().to_string()).color(Color32::WHITE), response.rect, Align2::RIGHT_BOTTOM);
        if self.tags.equipped.get() {
            text_in_rect(ui, tag_in_dungeon(), response.rect, Align2::LEFT_TOP);
        } else if self.tags.any_wardrobe() {
            text_in_rect(ui, tag_in_wardrobe(), response.rect, Align2::LEFT_TOP);
        }
        if let Some(image) = self.tags.rating().image() {
            let rect = Align2::RIGHT_TOP.align_size_within_rect(Rating::SIZE, response.rect);
            ui.put(rect, image);
        }
        if !self.attunements.is_empty() {            
            let rect = Align2::LEFT_BOTTOM.align_size_within_rect(Rating::SIZE, response.rect);
            ui.put(rect, forge::Tab::Attune.image().fit_to_exact_size(Rating::SIZE));
        }

        ui.advance_cursor_after_rect(response.rect);
        response
    }

    pub fn tooltip(&self, ui: &mut Ui) {
        // TODO consider switching the tooltip to using a LayoutJob
        // https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/misc_demo_window.rs#L568

        ui.style_mut().interaction.selectable_labels = true;

        // maybe show a name

        let mut any_info = false;
        if self.tags.equipped.get() {
            ui.horizontal(|ui| {
                ui.label(tag_in_dungeon());
                ui.label("equipped in active wardrobe")
            });
            any_info = true;
        } else if self.tags.any_wardrobe() {           
            ui.horizontal(|ui| { 
                ui.label(tag_in_wardrobe());
                let wardrobes = self.tags.wardrobes();
                if wardrobes.len() == 1 {
                    ui.label(format!("equipped in wardrobe {}", wardrobes[0] + 1))
                } else {
                    ui.label(format!("equipped in wardrobes {}", wardrobes.iter().map(|w| (w+1).to_string()).join(", ")))
                }

            });
            any_info = true;
        }
        if let Some(image) = self.tags.rating().image() {            
            ui.horizontal(|ui| {
                ui.add(image);
                let text: &'static str = self.tags.rating().into();
                ui.label(format!("marked as {}", text));
            });
            any_info = true;
        }
        if !self.attunements.is_empty() {                      
            ui.horizontal(|ui| {
                ui.add(forge::Tab::Attune.image().fit_to_exact_size(Rating::SIZE));
                let names = self.attunements.iter()
                    .map(|a| attune::name(a))
                    .join(", ");
                ui.label(format!("can be used as material to attune items to {}", names));

            });
            any_info = true;
        }

        if any_info {
            ui.separator();
        }

        if let Some(targeting) = self.targeting {
            use Targeting::*;
            match targeting {
                // Attack
                LowestHealth => ui.label("Target combatant with lowest current health"),
                LowestResistance(element) => ui.label(format!("Target combatant with lowest resistance to {:?}", element)),
                HighestMaxHealth => ui.label("Target combatant with highest maximum health"),
                HighestDamage => ui.label("Target combatant with highest damage"),
                RoundRobin(_) => ui.label("Target a different combatant every attack"),
                First => panic!(),

                // Defend
                Instant => ui.label("Triggers Instantly"),
                OnAttack => ui.label("Triggers when Attacked"),
            };
            ui.separator();
        }

        for (i, modifier) in self.mods.iter().enumerate() {
            ui.horizontal(|ui| {
                if self.rerolled_mod_idx == i as u8 {
                    ui.add(forge::Tab::RerollTarget.image().fit_to_exact_size(vec2(16., 16.)));
                }
                modifier.show_tooltip(ui);
            });
        }
    }
}

impl std::ops::Index<u8> for Item {
    type Output = RolledMod;
    fn index<'a>(&'a self, i: u8) -> &'a RolledMod {
        &self.mods[i as usize]
    }
}
impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Item {}

fn tag_in_dungeon() -> RichText {
    RichText::new("D").color(Color32::RED)
}
fn tag_in_wardrobe() -> RichText {
    RichText::new("W").color(Color32::YELLOW)
}