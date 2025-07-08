use std::rc::Rc;

use crate::prelude::*;
use crate::{equipment::equipment::Equip, explorer::Explorer, item::{Item, ItemRef, ItemType}, mods::atk_mod};
use super::equipment::FighterEquip;

#[apply(Default)]
pub struct Wardrobe {
    pub sets: [EquipmentSet; 9],
    pub equipped: usize,
}

#[apply(Default)]
pub struct EquipmentSet {
    pub fighter_equip: FighterEquip,
}

impl Wardrobe {
    pub fn equipped(&self) -> &EquipmentSet {
        &self.sets[self.equipped]
    }
    pub fn is_equipped(&self, idx: usize) -> bool {
        idx == self.equipped
    }
    pub fn set_equipped(&mut self, idx: usize) {
        if self.is_equipped(idx) {
            return;
        }

        self.sets[self.equipped]
            .iter()
            .filter_map(|i| i.upgrade())
            .for_each(|i| i.users.equipped.set(false));

        self.sets[idx]
            .iter()
            .filter_map(|i| i.upgrade())
            .for_each(|i| i.users.equipped.set(true));
        self.equipped = idx;
    }
}

impl EquipmentSet {
    pub fn iter(&self) -> impl Iterator<Item = &ItemRef> {
        self.fighter_equip.iter()
    }
    
    pub fn copy_owned(&self, owned: &mut Vec<Rc<Item>>) -> Self {
        Self { fighter_equip: self.fighter_equip.copy_owned(owned) }
    }
}

#[apply(Default)]
pub struct OwningEquipmentSet {
    pub equipment_set: EquipmentSet,
    _owned: Vec<Rc<Item>>,
}

impl From<&EquipmentSet> for OwningEquipmentSet {
    fn from(value: &EquipmentSet) -> Self {
        let mut owned = Vec::new();
        let equipment_set = value.copy_owned(&mut owned);
        Self { equipment_set, _owned: owned }
    }
}

#[apply(Enum)]
#[derive(Copy, PartialEq)]
pub enum ItemSlot {
    Weapon(usize),

    FighterShield,
    RangerQuiver,
    RangerSatchel,
    MageSupportGem,
    MageStaff,

    Helmet,
    Armor,
    Gloves,
    Ring(usize),
}
impl ItemSlot {
    pub fn accepts(&self, explorer: Explorer, item: &Item) -> bool {
        let t = item.item_type;
        use ItemSlot::*;
        match *self {
            Weapon(_) => {
                match explorer {
                    Explorer::Fighter => t == ItemType::Axe || t == ItemType::Sword,
                    Explorer::Ranger => t == ItemType::Crossbow || t == ItemType::Bow,
                    Explorer::Mage => todo!(),
                }
            }
            FighterShield => t == ItemType::Shield || (t == ItemType::Axe && item.has_mod(atk_mod::LIGHT.id)),
            RangerQuiver => todo!(),
            RangerSatchel => t == ItemType::Satchel,
            MageSupportGem => todo!(),
            MageStaff => todo!(),
            Helmet => t == ItemType::Helmet,
            Armor => t == ItemType::Armor,
            Gloves => t == ItemType::Gloves,
            Ring(_) => t == ItemType::Ring,
        }
    }
    pub fn default_type(&self, explorer: Explorer) -> Option<ItemType> {
        use ItemSlot::*;
        match *self {
            Weapon(_) => {
                match explorer {
                    Explorer::Fighter => Some(ItemType::Axe),
                    Explorer::Ranger => Some(ItemType::Bow),
                    Explorer::Mage => todo!(),
                }
                
            }
            FighterShield => Some(ItemType::Shield),
            RangerQuiver => todo!(),
            RangerSatchel => Some(ItemType::Satchel),
            MageSupportGem => todo!(),
            MageStaff => todo!(),
            Helmet => Some(ItemType::Helmet),
            Armor => Some(ItemType::Armor),
            Gloves => Some(ItemType::Gloves),
            Ring(_) => Some(ItemType::Ring),
        }
    }
}