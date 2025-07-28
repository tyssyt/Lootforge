use std::hash::Hash;

use crate::{mods::{atk_mod, char_mod, ModType}, prelude::*};

// TODO find a shorter way to do this, maybe macro?
pub static PENETRATION: AttuneGroup = AttuneGroup::of(AttuneKind::Element, &[
    &atk_mod::PENETRATION.bleed, &atk_mod::PENETRATION.fracture, &atk_mod::PENETRATION.madness, &atk_mod::PENETRATION.void
]);
pub static PEN_CONVERSION: AttuneGroup = AttuneGroup::of(AttuneKind::Element, &[
    &atk_mod::PEN_CONVERSION.bleed, &atk_mod::PEN_CONVERSION.fracture, &atk_mod::PEN_CONVERSION.madness, &atk_mod::PEN_CONVERSION.void
]);

pub static RES: AttuneGroup = AttuneGroup::of(AttuneKind::MatSpirit, &[
    &char_mod::MAT_RES, &char_mod::SPIRIT_RES
]);

#[apply(UnitEnum)]
#[derive(Hash)]
pub enum AttuneKind {
    Element,
    MatSpirit,
}
impl AttuneKind {
    pub const fn len(&self) -> usize {
        match self {
            AttuneKind::Element => 4,
            AttuneKind::MatSpirit => 2,
        }
    }
}

#[derive(Debug)]
pub struct AttuneGroup {
    pub kind: AttuneKind,
    mods: &'static [&'static ModType],
}
impl AttuneGroup {
    const fn of(kind: AttuneKind, mods: &'static [&'static ModType]) -> Self {
        if mods.len() != kind.len() {
            panic!("Invalid initialization of AttuneGroup!");
        }
        Self { kind, mods }
    }

    pub fn idx(&self, mod_type: &'static ModType) -> Option<usize> {
        self.mods.iter().position(|m| *m == mod_type)
    }
}
impl std::ops::Index<usize> for AttuneGroup {
    type Output = ModType;
    fn index(&self, i: usize) -> &'static ModType {
        self.mods[i]
    }
}
impl PartialEq for AttuneGroup {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}
impl Eq for &'static AttuneGroup {}

pub type Attunement = (AttuneKind, usize);

pub fn name(attunement: &Attunement) -> &'static str {
    // TODO color these
    match attunement {
        (AttuneKind::Element, 0) => "bleed",
        (AttuneKind::Element, 1) => "fracture",
        (AttuneKind::Element, 2) => "madness",
        (AttuneKind::Element, 3) => "void",
        (AttuneKind::MatSpirit, 0) => "material",
        (AttuneKind::MatSpirit, 1) => "spiritual",
        _ => panic!(),
    }
}