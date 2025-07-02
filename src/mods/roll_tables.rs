use std::{collections::BTreeMap, sync::LazyLock, iter};
use crate::{elemental::Elemental, prelude::*};
use super::{ModType, RolledMod};

use RollTableElement::*;
use super::atk_mod::*;
use super::def_mod::*;
use super::char_mod::*;

pub static ALL_MODS: LazyLock<BTreeMap<u16, &'static ModType>> = LazyLock::new(|| {
    let mods: Vec<_> = ALL_ROLL_TABLES.mods().collect();

    let duplicates: Vec<_> = mods.iter().duplicates_by(|m| m.id).collect();
    if duplicates.len() > 0 {
        panic!("Mod Ids have Duplicates : {:?},", duplicates)
    }

    mods.into_iter().map(|m| (m.id, m)).collect()
});

static ALL_ROLL_TABLES: RollTable = RollTable::new(
    &[
        Table(&SINGLE_TARGET_WEAPON_ROLL_TABLE),
        Table(&AOE_WEAPON_ROLL_TABLE),
        Table(&GLOVE_ROLL_TABLE),
        Table(&HELMET_ROLL_TABLE),
        Table(&ARMOR_ROLL_TABLE),
        Table(&RING_ROLL_TABLE),
    ],
    &[],
);

// item tables
pub static SINGLE_TARGET_WEAPON_ROLL_TABLE: RollTable = RollTable::new(
    &[
        Table(&WEAPON_ROLL_TABLE),
        Mod(&DEBUFF_OFF_ST, 0.8, true),
        Mod(&DEBUFF_UTIL_ST, 0.8, true),
    ],
    &[],
);
pub static AOE_WEAPON_ROLL_TABLE: RollTable = RollTable::new(
    &[
        Table(&WEAPON_ROLL_TABLE),
        Mod(&DEBUFF_OFF_AOE, 0.8, true),
        Mod(&DEBUFF_UTIL_AOE, 0.8, true),
    ],
    &[],
);

pub static GLOVE_ROLL_TABLE: RollTable = RollTable::new(
    &[
        Table(&OFFENSIVE_ROLL_TABLE),
    ],
    &[],
);

pub static HELMET_ROLL_TABLE: RollTable = RollTable::new(
    &[
        Table(&DEFENSIVE_ROLL_TABLE),
        Mod(&CDR, 1., true),
    ],
    &[],
);

pub static ARMOR_ROLL_TABLE: RollTable = RollTable::new(
    &[
        Table(&CHAR_ROLL_TABLE),
        Mod(&LIFESTEAL, 1., false),
        Mod(&SHIELDSTEAL, 1., false),
    ],
    &[
        &[&LIFESTEAL, &SHIELDSTEAL],
    ],
);
pub static RING_ROLL_TABLE: RollTable = RollTable::new(
    &[
        Table(&CHAR_ROLL_TABLE),
    ],
    &[],
);

// sub tables
static WEAPON_ROLL_TABLE: RollTable = RollTable::new(
    &[
        Table(&OFFENSIVE_ROLL_TABLE),
        Mod(&CULLING, 0.8, true),
        Mod(&CDR, 1., true),
    ],
    &[],
);

static OFFENSIVE_ROLL_TABLE: RollTable = RollTable::new(
    &[
        EMod(&ADDED_DMG, 2., false),
        EMod(&PENETRATION, 1., false),
    ],
    &[],
);
static DEFENSIVE_ROLL_TABLE: RollTable = RollTable::new(
    &[
        Mod(&SHIELD, 3., false),
        Mod(&HEAL, 3., false),
        Mod(&BLOCK, 1., true),
        Mod(&COUNTER, 1., true),
        Mod(&ATTUNE, 1.5, true),
        Mod(&REVERB, 1.5, true),
    ],
    &[
        &[&BLOCK, &COUNTER],
        &[&BLOCK, &ATTUNE],
        &[&BLOCK, &REVERB],
    ],
);
static CHAR_ROLL_TABLE: RollTable = RollTable::new(
    &[
        Mod(&HEALTH, 4., false),
        Mod(&PHY_RES, 2., false),
        Mod(&ELE_RES, 2., false),
        Mod(&HEAL_POWER, 2., false),
        Mod(&SHIELD_POWER, 2., false),
    ],
    &[],
);


pub struct RollTable {
    table: &'static [RollTableElement],    
    exclusive_groups: &'static [&'static [&'static ModType]],
    weight: f32,
}
impl RollTable {
    const fn new(table: &'static [RollTableElement], exclusive_groups: &'static [&'static [&'static ModType]]) -> Self {
        let mut weight = 0.;
        let mut i = 0;
        while i < table.len() { // hurray for const fn restrictions -.-
            weight += table[i].weight();
            i += 1;
        }
        Self {
            table,
            exclusive_groups,
            weight,
        }
    }

    pub fn roll_mod<R: Rng>(&self, rng: &mut R, existing_mods: &Vec<RolledMod>) -> &'static ModType {
        loop {
            let mod_ = self.choose_mod(rng);
            if self.check_mod_valid(mod_, existing_mods) {
                return mod_;
            }
        }
    }

    fn mods(&self) -> impl Iterator<Item = &'static ModType> {
        self.table.iter()
            .flat_map(|e| e.mods())
            .sorted_by_key(|m| m.id)
            .dedup_by(|&a, &b| ptr::eq(a, b))
    }

    fn choose_mod<R: Rng>(&self, rng: &mut R) -> &'static ModType {
        self.table.choose_weighted(rng, |e| e.weight()).unwrap().choose_mod(rng)
    }
    fn check_mod_valid(&self, mod_: &'static ModType, existing_mods: &Vec<RolledMod>) -> bool {
        if !self.table.iter().all(|e| e.check_mod_valid(mod_, existing_mods)) {
            return false;
        }
        if !self.exclusive_groups.iter().all(|g| check_exclusive_group_valid(mod_, existing_mods, g)) {
            return false;
        }
        true
    }
}

enum RollTableElement {
    Mod(&'static ModType, f32, bool),
    EMod(&'static Elemental<ModType>, f32, bool),
    Table(&'static RollTable),
}
impl RollTableElement {
    const fn weight(&self) -> f32 {
        match self {
            Mod(_, weight, _) => *weight,
            EMod(_, weight, _) => *weight,
            Table(roll_table) => roll_table.weight,
        }
    }
    
    fn mods(&self) -> Box<dyn Iterator<Item = &'static ModType>> {
        match self {
            Mod(mod_type, _, _) => Box::new(iter::once(*mod_type)),
            EMod(elemental, _, _) => Box::new(elemental.iter()),
            Table(roll_table) => Box::new(roll_table.mods()),
        }
    }
    
    fn choose_mod<R: Rng>(&self, rng: &mut R) -> &'static ModType {
        match self {
            Mod(mod_type, _, _) => mod_type,
            EMod(elemental, _, _) => elemental.choose(rng),
            Table(roll_table) => roll_table.choose_mod(rng),
        }
    }
    fn check_mod_valid(&self, mod_: &'static ModType, existing_mods: &Vec<RolledMod>) -> bool {
        match self {
            Mod(mod_type, _, unique) => !unique || mod_.id != mod_type.id || existing_mods.iter().all(|m| m.mod_id != mod_.id) ,
            EMod(elemental, _, unique) => !unique || elemental.iter().all(|m| mod_.id != m.id) || elemental.iter().all(|e| existing_mods.iter().all(|m| m.mod_id != e.id)),
            Table(roll_table) => roll_table.check_mod_valid(mod_, existing_mods),
        }
    }
}

fn check_exclusive_group_valid(mod_: &ModType, existing_mods: &Vec<RolledMod>, group: &[&ModType]) -> bool {
    if !group.contains(&mod_) { return true; }

    for existing_mod in existing_mods {
        if existing_mod.mod_id == mod_.id { continue; }
        if group.contains(&existing_mod.mod_type()) { return false; }
    }
    true
}
