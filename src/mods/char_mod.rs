use super::ModType;
use crate::elemental::Element;
use crate::equipment::equipment::Equip;
use crate::mods::attune;
use crate::prelude::*;

pub static HEALTH: ModType = ModType {
    id: 50,
    prefix_name: "stout",
    roll_range: 250..=500,
    attune: None,
    show_tooltip: |this, ui, roll| {
        tooltip!("Increase max health by %roll %range");
    },
    register: |hooks, _item, _equip, roll| {
        hooks.on_char(move |char| char.max_health += roll as f32);
    },
};
pub static MAT_RES: ModType = ModType {
    id: 51,
    prefix_name: "heavy",
    roll_range: 15..=30,
    attune: Some(&attune::RES),
    show_tooltip: |this, ui, roll| {
        tooltip!("Increase Material (%bleed & %fracture) resistance by %roll %range");
    },
    register: |hooks, _item, _equip, roll| {
        hooks.on_char(move |char| {
            char.resistances.bleed += roll as f32;
            char.resistances.fracture += roll as f32;
        });
    },
};
pub static SPIRIT_RES: ModType = ModType {
    id: 52,
    prefix_name: "engraved",
    roll_range: 15..=30,
    attune: Some(&attune::RES),
    show_tooltip: |this, ui, roll| {
        tooltip!("Increase Spiritual (%madness & %void) resistance by %roll %range");
    },
    register: |hooks, _item, _equip, roll| {
        hooks.on_char(move |char| {
            char.resistances.madness += roll as f32;
            char.resistances.void += roll as f32;
        });
    },
};
pub static CDR: ModType = ModType {
    id: 53,
    prefix_name: "quick",
    roll_range: 0..=0,
    attune: None,
    show_tooltip: |_this, ui, _roll| {
        tooltip!("Skills cooldown is reduce by 10%");
    },
    register: |hooks, _item, _equip, _roll| {
        hooks.on_char(move |char| char.cdr += 10);
    },
};
pub static HEAL_POWER: ModType = ModType {
    id: 54,
    prefix_name: "vitalising",
    roll_range: 20..=30,
    attune: None,
    show_tooltip: |this, ui, roll| {
        tooltip!("Increase heal power by %roll% %range");
    },
    register: |hooks, _item, _equip, roll| {
        hooks.on_char(move |char| char.heal_power += percent(roll));
    },
};
pub static SHIELD_POWER: ModType = ModType {
    id: 55,
    prefix_name: "aegised",
    roll_range: 20..=50,
    attune: None,
    show_tooltip: |this, ui, roll| {
        tooltip!("Increase shield power by %roll% %range");
    },
    register: |hooks, _item, _equip, roll| {
        hooks.on_char(move |char| char.shield_power += percent(roll));
    },
};
pub static SHIELD_START: ModType = ModType {
    id: 56,
    prefix_name: "bulwark",
    roll_range: 30..=50,
    attune: None,
    show_tooltip: |this, ui, roll| {
        tooltip!("Start combat with %roll% %range of your max health as shield");
    },
    register: |hooks, _item, _equip, roll| {
        hooks.on_combat_start(move |effects, _char| effects.shield_from_max_health += percent(roll));
    },
};
pub static HEALTH_EX: ModType = ModType {
    id: 57,
    prefix_name: "immortal",
    roll_range: 0..=0,
    attune: None,
    show_tooltip: |_this, ui, _roll| {
        tooltip!("Increase max health by 1234, if no other items give max health");
    },
    register: |hooks, item, equip, _roll| {
        let has_health = equip.iter()
            .filter_map(|i| i.upgrade())
            .filter(|i| i.id != item.id)
            .any(|i| i.has_mod(HEALTH.id) || i.has_mod(HEALTH_EX.id));

        if !has_health {
            hooks.on_char(move |char| char.max_health += 1234.);
        }
    },
};

fn percent(roll: u16) -> f32 {
    (roll as f32) / 100.
}
