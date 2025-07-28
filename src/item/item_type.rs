use enumset::EnumSetType;

use crate::prelude::*;
use crate::mods::{atk_mod, def_mod, RolledMod};
use crate::mods::roll_tables::*;

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
            Axe => include_image!("../../assets/items/axe.png"),
            Sword => include_image!("../../assets/items/sword.png"),
            Shield => include_image!("../../assets/items/shield.png"),
            Crossbow => include_image!("../../assets/items/hand_crossbow.png"),
            Bow => include_image!("../../assets/items/bow.png"),
            Satchel => include_image!("../../assets/items/satchel.png"),
            Armor => include_image!("../../assets/items/armor.png"),
            Helmet => include_image!("../../assets/items/helmet.png"),
            Gloves => include_image!("../../assets/items/gloves.png"),
            Ring => include_image!("../../assets/items/ring.png"),
        };
        Image::new(source).fit_to_exact_size(Self::SIZE)
    }
}