use std::rc::Rc;

use crate::item::{Item, ItemRef, ItemType};
use super::equipment::FighterEquip;

#[derive(Default)]
pub struct Wardrobe {
    pub sets: [EquipmentSet; 9],
    pub equipped: usize,
}

#[derive(Default)]
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

#[derive(Default)]
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ItemSlot {
    FighterWeapon(usize),
    FighterShield,

    RangerWeapon,
    RangerQuiver,
    RangerSatchel,

    MageAttackGem,
    MageSupportGem,
    MageStaff,

    Helmet,
    Armor,
    Gloves,
    Ring(usize),
}
impl ItemSlot {
    pub fn accepts(&self, item: &Item) -> bool {
        let t = item.item_type;
        use ItemSlot::*;
        match *self {
            FighterWeapon(_) => t == ItemType::Axe || t == ItemType::Sword,
            FighterShield => t == ItemType::Shield, // TODO or Sword with light modifier
            RangerWeapon => t == ItemType::Crossbow || t == ItemType::Bow,
            RangerQuiver => todo!(),
            RangerSatchel => t == ItemType::Satchel,
            MageAttackGem => todo!(),
            MageSupportGem => todo!(),
            MageStaff => todo!(),
            Helmet => t == ItemType::Helmet,
            Armor => t == ItemType::Armor,
            Gloves => t == ItemType::Gloves,
            Ring(_) => t == ItemType::Ring,
        }
    }
    pub fn default_type(&self) -> Option<ItemType> {
        use ItemSlot::*;
        match *self {
            FighterWeapon(_) => Some(ItemType::Axe),
            FighterShield => Some(ItemType::Shield),
            RangerWeapon => todo!(), // crossbow
            RangerQuiver => todo!(),
            RangerSatchel => Some(ItemType::Satchel),
            MageAttackGem => todo!(),
            MageSupportGem => todo!(),
            MageStaff => todo!(),
            Helmet => Some(ItemType::Helmet),
            Armor => Some(ItemType::Armor),
            Gloves => Some(ItemType::Gloves),
            Ring(_) => Some(ItemType::Ring),
        }
    }
}