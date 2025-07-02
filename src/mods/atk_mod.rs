use crate::prelude::*;
use crate::elemental::{Elemental, Element};
use crate::combat::buff::Debuff;
use super::ModType;

pub static ADDED_DMG: Elemental<ModType> = Elemental {
    bleed: ModType {
        id: 1,
        prefix_name: "lacerating",
        roll_range: 20..=50,
        show_tooltip: |this, ui, roll| { tooltip!("Add %roll %range %bleed Damage"); },
        register: |hooks, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.damage.bleed += roll as f32);
        },
    },
    fracture: ModType {
        id: 2,
        prefix_name: "shattering",
        roll_range: 20..=50,
        show_tooltip: |this, ui, roll| { tooltip!("Add %roll %range %fracture Damage"); },
        register: |hooks, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.damage.fracture += roll as f32);
        },
    },
    madness: ModType {
        id: 3,
        prefix_name: "maddening",
        roll_range: 20..=50,
        show_tooltip: |this, ui, roll| { tooltip!("Add %roll %range %madness Damage"); },
        register: |hooks, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.damage.madness += roll as f32);
        },
    },
    void: ModType {
        id: 4,
        prefix_name: "empty",
        roll_range: 20..=50,
        show_tooltip: |this, ui, roll| { tooltip!("Add %roll %range %void Damage"); },
        register: |hooks, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.damage.void += roll as f32);
        },
    },
};

pub static PENETRATION: Elemental<ModType> = Elemental {
    bleed: ModType {
        id: 5,
        prefix_name: "sharp",
        roll_range: 10..=20,
        show_tooltip: |this, ui, roll| { tooltip!("Overcome %roll %range %bleed Resistance"); },
        register: |hooks, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.penetration.bleed += roll as f32);
        },
    },
    fracture: ModType {
        id: 6,
        prefix_name: "grinding",
        roll_range: 10..=20,
        show_tooltip: |this, ui, roll| { tooltip!("Overcome %roll %range %fracture Resistance"); },
        register: |hooks, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.penetration.fracture += roll as f32);
        },
    },
    madness: ModType {
        id: 7,
        prefix_name: "disturbing",
        roll_range: 10..=20,
        show_tooltip: |this, ui, roll| { tooltip!("Overcome %roll %range %madness Resistance"); },
        register: |hooks, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.penetration.madness += roll as f32);
        },
    },
    void: ModType {
        id: 8,
        prefix_name: "silent", // TODO can do better hollow, vast eternal, perpetual
        roll_range: 10..=20,
        show_tooltip: |this, ui, roll| { tooltip!("Overcome %roll %range %void Resistance"); },
        register: |hooks, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.penetration.void += roll as f32);
        },
    },
};

pub static CULLING: ModType = ModType {
    id: 9,
    prefix_name: "culling",
    roll_range: 5..=10,
    show_tooltip: |this, ui, roll| { tooltip!("Kill enemies under %roll% %range of their max HP"); },
    register: |hooks, roll| {
        hooks.on_post_hit(move |attack, _skill, _user, _target, _hit| attack.cull_threshhold = (roll as f32).at_least(attack.cull_threshhold));
    },
};

pub static LIFESTEAL: ModType = ModType {
    id: 10,
    prefix_name: "vampiric",
    roll_range: 4..=8,
    show_tooltip: |this, ui, roll| { tooltip!("Steal %roll% %range of Damage dealt as Life"); },
    register: |hooks, roll| {
        hooks.on_post_hit(move |attack, _skill, _user, _target, _hit| attack.life_steal += (roll as f32) / 100.);
    },
};
pub static SHIELDSTEAL: ModType = ModType {
    id: 11,
    prefix_name: "leeching",
    roll_range: 5..=15,
    show_tooltip: |this, ui, roll| { tooltip!("Steal %roll% %range of Damage dealt as Shield"); },
    register: |hooks, roll| {
        hooks.on_post_hit(move |attack, _skill, _user, _target, _hit| attack.shield_steal += (roll as f32) / 100.);
    },
};

// TODO more details: primary damage type, all debuffs
pub static DEBUFF_OFF_ST: ModType = ModType {
    id: 12,
    prefix_name: "afflicting",
    roll_range: 0..=0,
    show_tooltip: |_this, ui, _roll| { tooltip!("Every 2nd attack inflicts the offensive Debuff of the primary damage type"); },
    register: |hooks, _roll| {
        hooks.on_post_hit(move |attack, skill, _user, _target, hit| {
            if skill.uses % 2 == 0 {
                match hit.pre_res_dmg.max_idx() {
                    Element::Bleed    => attack.debuffs.push(Debuff::bleed()),
                    Element::Fracture => attack.debuffs.push(Debuff::vulnerable()),
                    Element::Madness  => attack.debuffs.push(Debuff::confused()),
                    Element::Void     => attack.debuffs.push(Debuff::echo(hit.pre_res_dmg)),
                }            
            }
        });
    },
};
pub static DEBUFF_OFF_AOE: ModType = ModType {
    id: 13,
    prefix_name: "afflicting",
    roll_range: 0..=0,
    show_tooltip: |_this, ui, _roll| { tooltip!("Every 3rd attack inflicts the offensive Debuff of the primary damage type"); },
    register: |hooks, _roll| {
        hooks.on_post_hit(move |attack, skill, _user, _target, hit| {
            if skill.uses % 3 == 0 {
                match hit.pre_res_dmg.max_idx() {
                    Element::Bleed    => attack.debuffs.push(Debuff::bleed()),
                    Element::Fracture => attack.debuffs.push(Debuff::vulnerable()),
                    Element::Madness  => attack.debuffs.push(Debuff::confused()),
                    Element::Void     => attack.debuffs.push(Debuff::echo(hit.pre_res_dmg)),
                }            
            }
        });
    },
};

// TODO more details: primary damage type, all debuffs
pub static DEBUFF_UTIL_ST: ModType = ModType {
    id: 14,
    prefix_name: "impairing",
    roll_range: 0..=0,
    show_tooltip: |_this, ui, _roll| { tooltip!("Every 2nd attack gives the enemy the utility debuff of the primary damage type"); },
    register: |hooks, _roll| {
        hooks.on_post_hit(move |attack, skill, _user, _target, hit| {
            if skill.uses % 2 == 0 {
                match hit.pre_res_dmg.max_idx() {
                    Element::Bleed    => attack.debuffs.push(Debuff::lifelink()),
                    Element::Fracture => attack.debuffs.push(Debuff::incapacitated()),
                    Element::Madness  => attack.debuffs.push(Debuff::dazed()),
                    Element::Void     => attack.debuffs.push(Debuff::soullink()),
                }            
            }
        });
    },
};
pub static DEBUFF_UTIL_AOE: ModType = ModType {
    id: 15,
    prefix_name: "impairing",
    roll_range: 0..=0,
    show_tooltip: |_this, ui, _roll| { tooltip!("Every 3rd attack gives the enemy the utility debuff of the primary damage type"); },
    register: |hooks, _roll| {
        hooks.on_post_hit(move |attack, skill, _user, _target, hit| {
            if skill.uses % 3 == 0 {
                match hit.pre_res_dmg.max_idx() {
                    Element::Bleed    => attack.debuffs.push(Debuff::lifelink()),
                    Element::Fracture => attack.debuffs.push(Debuff::incapacitated()),
                    Element::Madness  => attack.debuffs.push(Debuff::dazed()),
                    Element::Void     => attack.debuffs.push(Debuff::soullink()),
                }
            }
        });
    },
};