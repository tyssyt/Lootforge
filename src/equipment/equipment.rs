use std::array;

use crate::prelude::*;
use crate::item::{Item, ItemRef, ItemUsers};

use super::wardrobe::ItemSlot;

#[derive(Debug, Default)]
pub struct FighterEquip {
    pub weapons: [ItemRef; 2],
    pub shield: ItemRef,

    pub common: CommonEquip<3>,
}
impl FighterEquip {
    pub fn iter(&self) -> impl Iterator<Item = &ItemRef> {
        [&self.weapons[0], &self.weapons[1], &self.shield]
            .into_iter()
            .chain(self.common.iter())
    }

    pub fn get_item(&self, slot: ItemSlot) -> (ItemRef, bool) {
        match slot {
            // for weapon 1, if not present check if weapon0 2 handed and set return bool to true
            ItemSlot::FighterWeapon(0) => (self.weapons[0].clone(), false),
            ItemSlot::FighterWeapon(1) => {
                if self.weapons[0]
                    .upgrade()
                    .filter(|i| i.item_type.two_handed())
                    .is_some()
                {
                    (self.weapons[0].clone(), true)
                } else {
                    (self.weapons[1].clone(), false)
                }
            }
            ItemSlot::FighterWeapon(_) => panic!(),
            ItemSlot::FighterShield => (self.shield.clone(), false),
            ItemSlot::RangerWeapon => todo!(),
            ItemSlot::RangerQuiver => todo!(),
            ItemSlot::RangerSatchel => todo!(),
            ItemSlot::MageAttackGem => todo!(),
            ItemSlot::MageSupportGem => todo!(),
            ItemSlot::MageStaff => todo!(),
            ItemSlot::Helmet => (self.common.helmet.clone(), false),
            ItemSlot::Armor => (self.common.armor.clone(), false),
            ItemSlot::Gloves => (self.common.gloves.clone(), false),
            ItemSlot::Ring(i) => (self.common.rings[i].clone(), false),
        }
    }

    pub fn set_item(&mut self, item: Rc<Item>, slot: ItemSlot) -> Option<EquipChange> {
        match slot {
            ItemSlot::FighterWeapon(i) => set_two_handed(item, &mut self.weapons, i),
            ItemSlot::FighterShield => Some(set_item(item, &mut self.shield)),
            ItemSlot::Helmet => Some(set_item(item, &mut self.common.helmet)),
            ItemSlot::Armor => Some(set_item(item, &mut self.common.armor)),
            ItemSlot::Gloves => Some(set_item(item, &mut self.common.gloves)),
            ItemSlot::Ring(i) => set_item_array(item, &mut self.common.rings, i),
            ItemSlot::RangerWeapon
            | ItemSlot::RangerQuiver
            | ItemSlot::RangerSatchel
            | ItemSlot::MageAttackGem
            | ItemSlot::MageSupportGem
            | ItemSlot::MageStaff => panic!(),
        }
    }    

    pub fn copy_owned(&self, owned: &mut Vec<Rc<Item>>) -> Self {
        Self {
            weapons: array::from_fn(|i| copy_owned(&self.weapons[i], owned)),
            shield:  copy_owned(&self.shield, owned),
            common:  self.common.copy_owned(owned),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommonEquip<const RINGS: usize> {
    pub helmet: ItemRef,
    pub armor: ItemRef,
    pub gloves: ItemRef,
    pub rings: [ItemRef; RINGS],
}
impl<const RINGS: usize> Default for CommonEquip<RINGS> {
    fn default() -> Self {
        Self {
            helmet: Default::default(),
            armor: Default::default(),
            gloves: Default::default(),
            rings: [const { Weak::new() }; RINGS],
        }
    }
}
impl<const RINGS: usize> CommonEquip<RINGS> {
    pub fn iter(&self) -> impl Iterator<Item = &ItemRef> {
        [&self.helmet, &self.armor, &self.gloves]
            .into_iter()
            .chain(self.rings.iter())
    }

    pub fn copy_owned(&self, owned: &mut Vec<Rc<Item>>) -> Self {
        Self {
            helmet: copy_owned(&self.helmet, owned),
            armor:  copy_owned(&self.armor, owned),
            gloves: copy_owned(&self.gloves, owned),
            rings:  array::from_fn(|i| copy_owned(&self.rings[i], owned)),
        }
    }
}

fn set_two_handed(item: Rc<Item>, hands: &mut [ItemRef; 2], i: usize) -> Option<EquipChange> {
    if item.item_type.two_handed() {
        let mut change = set_item(item, &mut hands[0]);
        change.removed2 = hands[1].clone();
        hands[1] = Weak::new();
        Some(change)
    } else if i == 1 && hands[0].upgrade().is_some_and(|w| w.item_type.two_handed()) {
        let mut change = set_item(item, &mut hands[1]);
        change.removed2 = hands[0].clone();
        hands[0] = Weak::new();
        Some(change)
    } else {
        set_item_array(item, hands, i)
    }
}

fn set_item_array(item: Rc<Item>, array: &mut [ItemRef], i: usize) -> Option<EquipChange> {
    let item_as_weak = Rc::downgrade(&item);
    if let Some(old) = array.iter().position(|r| r.ptr_eq(&item_as_weak)) {
        array.swap(old, i);
        None
    } else {
        Some(set_item(item, &mut array[i]))
    }
}

fn set_item(new: Rc<Item>, old: &mut ItemRef) -> EquipChange {
    let old_clone = old.clone();
    *old = Rc::downgrade(&new);
    EquipChange {
        added: new,
        removed: old_clone,
        removed2: Default::default(),
    }
}

pub struct EquipChange {
    pub added: Rc<Item>,
    pub removed: ItemRef,
    pub removed2: ItemRef,
}

fn copy_owned(item: &ItemRef, owned: &mut Vec<Rc<Item>>) -> ItemRef {
    if let Some(item) = item.upgrade() {
        let cloned = Rc::new( Item {            
            id: item.id,
            item_type: item.item_type,
            targeting: item.targeting,
            mods: item.mods.clone(),
            users: ItemUsers::default(),
        });

        let weak = Rc::downgrade(&cloned);
        owned.push(cloned);
        weak
    } else {
        Default::default()
    }
}