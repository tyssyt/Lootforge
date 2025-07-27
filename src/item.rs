use enumset::EnumSetType;

use crate::mods::attune::AttuneKind;
use crate::mods::roll_tables::*;
use crate::panels::forge::forge;
use crate::prelude::*;

use crate::{
    combat::skill::{targeting::Targeting},
    mods::*,
    widgets::text_in_rect::text_in_rect,
};

#[repr(u8)]
#[apply(UnitEnum)]
#[derive(EnumSetType)]
#[enumset(no_super_impls)]
pub enum ItemType {
    // Fighter
    Axe = 1,
    Sword = 2,
    Shield = 3,

    // Ranger
    Crossbow = 4,
    Bow = 5,
    Satchel = 6,
    // Quiver = 7

    // Mage

    // 3 Skillgems 8,9,10
    // Staff 11

    // Common
    Armor = 12,
    Helmet = 13,
    Gloves = 14,
    Ring = 15,
}

#[derive(Debug)]
pub struct Item {
    pub id: usize,
    pub item_type: ItemType,
    // TODO most items don't even have targeting, but I do need something to store inherit stuff, prolly in itemtype data
    pub targeting: Option<Targeting>,
    pub mods: Vec<RolledMod>,
    pub rerolled_mod_idx: u8,
    // TODO most items will not have users, so we could make this Option<Box> or something to reduce size
    pub users: ItemUsers,
}

pub type ItemRef = Weak<Item>;

impl ItemType {
    pub fn roll_mod(&self, rng: &mut impl Rng, mods: &Vec<RolledMod>) -> RolledMod {
        use ItemType::*;
        if mods.is_empty() {            
            match *self {
                Axe | Sword | Bow | Crossbow => {
                    if rng.random() {
                        return atk_mod::ADDED_DMG.bleed.roll(rng)
                    } else {                        
                        return atk_mod::ADDED_DMG.fracture.roll(rng)
                    }
                },
                Helmet => return def_mod::SHIELD.roll(rng),
                _ => {}
            }
        }

        match *self {
            Axe => AXE_ROLL_TABLE.roll_mod(rng, mods).roll(rng), 
            Sword => SWORD_ROLL_TABLE.roll_mod(rng, mods).roll(rng),
            Crossbow | Bow => panic!(), // attack gems

            Gloves => GLOVE_ROLL_TABLE.roll_mod(rng, mods).roll(rng),

            Shield | Satchel => SHIELD_ROLL_TABLE.roll_mod(rng, mods).roll(rng), // util gem
            Helmet => HELMET_ROLL_TABLE.roll_mod(rng, mods).roll(rng),

            Armor =>  ARMOR_ROLL_TABLE.roll_mod(rng, mods).roll(rng),
            Ring =>  RING_ROLL_TABLE.roll_mod(rng, mods).roll(rng),
        }
    }

    pub fn two_handed(&self) -> bool {
        use ItemType::*;
        match *self {
            Sword | Bow => true, // TODO add aoe gem
            _ => false,
        }
    }

    pub fn has_targeting(&self) -> bool {
        use ItemType::*;
        match *self {
            Satchel | Ring => true,
            _ => false,
        }
    }

    pub const SIZE: Vec2 = vec2(64., 64.);
    pub fn image(&self) -> Image<'_> {
        use ItemType::*;
        let source = match *self {
            Axe => include_image!("../assets/items/axe.png"),
            Sword => include_image!("../assets/items/sword.png"),
            Shield => include_image!("../assets/items/shield.png"),
            Crossbow => include_image!("../assets/items/hand_crossbow.png"),
            Bow => include_image!("../assets/items/bow.png"),
            Satchel => include_image!("../assets/items/satchel.png"),
            Armor => include_image!("../assets/items/armor.png"),
            Helmet => include_image!("../assets/items/helmet.png"),
            Gloves => include_image!("../assets/items/gloves.png"),
            Ring => include_image!("../assets/items/ring.png"),
        };
        Image::new(source).fit_to_exact_size(Self::SIZE)
    }
}

#[derive(Debug, Default)]
pub struct ItemUsers {
    pub equipped: Cell<bool>,
    pub any_wardrobe: Cell<bool>,
    wardrobe: [Cell<bool>; 9], // more precise info, like which char and then the exact slot?
                               // in workbench (store which position?)
}
impl ItemUsers {
    pub fn add_wardrobe(&self, i: usize) {
        self.wardrobe[i].set(true);
        self.any_wardrobe.set(true);
    }
    pub fn remove_wardrobe(&self, i: usize) {
        self.wardrobe[i].set(false);
        if self.wardrobe.iter().all(|b| !b.get()) {
            self.any_wardrobe.set(false);
        }
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Item {}

impl Item {
    pub fn new(item_type: ItemType, mods: Vec<RolledMod>, targeting: Option<Targeting>) -> Self {
        Self {
            id: 0,
            item_type,
            targeting,
            mods,
            rerolled_mod_idx: u8::MAX,
            users: ItemUsers::default(),
        }
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

    pub fn attunements(&self) -> Vec<(AttuneKind, usize)> {
        self.mods.iter()
            .filter_map(|m| m.mod_type().attunement())
            .map(|(group, idx )| (group.kind, idx))
            .into_group_map()
            .iter()
            .filter(|(_, idx)| idx.iter().all_equal())
            .map(|(kind, idx)| (*kind, idx[0]))
            .collect()
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

        text_in_rect(
            ui,
            self.rank().to_string(),
            Color32::WHITE,
            response.rect,
            Align2::RIGHT_TOP,
        );
        if self.users.equipped.get() {
            text_in_rect(ui, "D", Color32::RED, response.rect, Align2::LEFT_TOP);
        } else if self.users.any_wardrobe.get() {
            text_in_rect(ui, "W", Color32::YELLOW, response.rect, Align2::LEFT_TOP);
        }

        response
    }

    pub fn tooltip(&self, ui: &mut Ui) {
        // TODO consider switching the tooltip to using a LayoutJob
        // https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/misc_demo_window.rs#L568

        ui.style_mut().interaction.selectable_labels = true;

        // maybe show a name

        if let Some(targeting) = self.targeting {
            use Targeting::*;
            match targeting {
                // Attack
                LowestHealth => {
                    ui.label("Target lowest health");
                }
                LowestEffectiveHealth(element) => {
                    ui.label(format!(
                        "Target lowest effective health against {:?}",
                        element
                    ));
                }
                LowestResistance(element) => {
                    ui.label(format!("Target lowest resistance to {:?}", element));
                }
                HighestRank => todo!(),
                LowestRank => todo!(),
                RoundRobin(_) => {
                    ui.label("Target Round Robin");
                }
                First => panic!(),

                // Defend
                Instant => {
                    ui.label("Triggers Instantly");
                }
                OnAttack => {
                    ui.label("Triggers when Attacked");
                }
            }
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

    // we could have a fun namebuilder, like *pre* *item* of *pre* *suf* & *pre* *suf*
}
impl std::ops::Index<u8> for Item {
    type Output = RolledMod;
    fn index<'a>(&'a self, i: u8) -> &'a RolledMod {
        &self.mods[i as usize]
    }
}