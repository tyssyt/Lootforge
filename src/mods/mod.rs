use roll_tables::ALL_MODS;

use crate::{combat::hooks::CombatHooks, equipment::equipment::EquipEnum, item::Item, prelude::*};

pub mod atk_mod;
pub mod char_mod;
pub mod def_mod;
pub mod roll_tables;

#[derive(Debug, Clone)]
pub struct RolledMod {
    pub mod_id: u16,
    pub roll: u16,
}

impl RolledMod {
    pub fn mod_type(&self) -> &'static ModType {
        ALL_MODS[&self.mod_id]
    }
    pub fn show_tooltip(&self, ui: &mut Ui) {
        // TODO change default color to a slightly brighter grey
        // TODO check how poe and d3/4 show mods
        // TODO obv find a better way to do this per mod lul
        (self.mod_type().show_tooltip)(&self.mod_type(), ui, self.roll);
    }
    pub fn register(&self, hooks: &mut CombatHooks, item: &Item, equip: &EquipEnum) {
        (self.mod_type().register)(hooks, item, equip, self.roll);
    }
}

// TODO why is this not an enum?
// well I want all of them to be 'static
// but instead they can be an owned ModType which references the sub modtype with a 'static
// but we need to keep in mind size, I want items to be as small as possible to save ram, and there will be thousands of mods
// right now its the size if a pointer, that would add enum overhead
// alternativly, I could just store the mod id, not like its read often (and the dungeon and store a ref)


pub struct ModType {
    pub id: u16,
    pub prefix_name: &'static str,
    pub roll_range: RangeInclusive<u16>,

    pub show_tooltip: fn(&Self, &mut Ui, u16), // potentially, we can do this as a string with placeholder and style info... but lets get it decent for a good number of mods before we begin that
    // longer description for book
    pub register: fn(hooks: &mut CombatHooks, item: &Item, equip: &EquipEnum, roll: u16),
}

impl PartialEq for ModType {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}
impl Eq for &'static ModType {}

impl std::fmt::Debug for ModType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModType")
        .field("name", &self.prefix_name)
        .field("id", &self.id)
        .finish()
    }
}

impl ModType {
    pub fn roll(&'static self, rng: &mut impl Rng) -> RolledMod {
        RolledMod {
            mod_id: self.id,
            roll: rng.random_range(self.roll_range.clone()), //TODO understand why clone is necessary
        }
    }
}