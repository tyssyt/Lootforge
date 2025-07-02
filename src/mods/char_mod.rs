use crate::prelude::*;
use crate::elemental::Element;
use super::ModType;

pub static HEALTH: ModType = ModType {
    id: 20,
    prefix_name: "stout",
    roll_range: 250..=500,
    show_tooltip: |this, ui, roll| { tooltip!("Increase max health by %roll %range"); },
    register: |hooks, roll| {
        hooks.on_char(move |char| char.max_health += roll as f32);
    }
};
pub static PHY_RES: ModType = ModType {
    id: 21,
    prefix_name: "heavy",
    roll_range: 15..=30,
    show_tooltip: |this, ui, roll| { tooltip!("Increase Material (%bleed & %fracture) resistance by %roll %range"); },
    register: |hooks, roll| {
        hooks.on_char(move |char| {char.resistances.bleed += roll as f32; char.resistances.fracture += roll as f32;});
    }
};
pub static ELE_RES: ModType = ModType {
    id: 22,
    prefix_name: "engraved",
    roll_range: 15..=30,
    show_tooltip: |this, ui, roll| { tooltip!("Increase Spiritual (%madness & %void) resistance by %roll %range"); },
    register: |hooks, roll| {
        hooks.on_char(move |char| {char.resistances.madness += roll as f32; char.resistances.void += roll as f32;});
    },
};
pub static CDR: ModType = ModType {
    id: 23,
    prefix_name: "quick",
    roll_range: 0..=0,
    show_tooltip: |_this, ui, _roll| { tooltip!("Skills cooldown is reduce by 10%"); },
    register: |hooks, _roll| {
        hooks.on_char(move |char| char.cdr += 10);
    },
};
pub static HEAL_POWER: ModType = ModType {
    id: 24,
    prefix_name: "vitalising",
    roll_range: 20..=30,
    show_tooltip: |this, ui, roll| { tooltip!("Increase heal power by %roll% %range"); },
    register: |hooks, roll| {
        hooks.on_char(move |char| char.heal_power += (roll as f32) / 100.);
    },
};
pub static SHIELD_POWER: ModType = ModType {
    id: 25,
    prefix_name: "aegised",
    roll_range: 20..=50,
    show_tooltip: |this, ui, roll| { tooltip!("Increase shield power by %roll% %range"); },
    register: |hooks, roll| {
        hooks.on_char(move |char| char.shield_power += (roll as f32) / 100.);
    },    
};