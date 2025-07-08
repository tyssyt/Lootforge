use crate::combat::skill::skill::SkillSource;
use crate::equipment::equipment::Equip;
use crate::equipment::wardrobe::ItemSlot;
use crate::mods::attune;
use crate::prelude::*;
use crate::elemental::{Elemental, Element};
use crate::combat::buff::Debuff;
use super::ModType;

pub static ADDED_DMG: Elemental<ModType> = Elemental {
    bleed: ModType {
        id: 1,
        prefix_name: "lacerating",
        roll_range: 25..=50,
        attune: None,
        show_tooltip: |this, ui, roll| { tooltip!("Add %roll %range %bleed Damage"); },
        register: |hooks, _item, _equip, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.damage.bleed += roll as f32);
        },
    },
    fracture: ModType {
        id: 2,
        prefix_name: "shattering",
        roll_range: 25..=50,
        attune: None,
        show_tooltip: |this, ui, roll| { tooltip!("Add %roll %range %fracture Damage"); },
        register: |hooks, _item, _equip, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.damage.fracture += roll as f32);
        },
    },
    madness: ModType {
        id: 3,
        prefix_name: "maddening",
        roll_range: 25..=50,
        attune: None,
        show_tooltip: |this, ui, roll| { tooltip!("Add %roll %range %madness Damage"); },
        register: |hooks, _item, _equip, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.damage.madness += roll as f32);
        },
    },
    void: ModType {
        id: 4,
        prefix_name: "empty",
        roll_range: 25..=50,
        attune: None,
        show_tooltip: |this, ui, roll| { tooltip!("Add %roll %range %void Damage"); },
        register: |hooks, _item, _equip, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.damage.void += roll as f32);
        },
    },
};

pub static PENETRATION: Elemental<ModType> = Elemental {
    bleed: ModType {
        id: 5,
        prefix_name: "sharp",
        roll_range: 10..=20,
        attune: Some(&attune::PENETRATION),
        show_tooltip: |this, ui, roll| { tooltip!("Overcome %roll %range %bleed Resistance"); },
        register: |hooks, _item, _equip, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.penetration.bleed += roll as f32);
        },
    },
    fracture: ModType {
        id: 6,
        prefix_name: "grinding",
        roll_range: 10..=20,
        attune: Some(&attune::PENETRATION),
        show_tooltip: |this, ui, roll| { tooltip!("Overcome %roll %range %fracture Resistance"); },
        register: |hooks, _item, _equip, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.penetration.fracture += roll as f32);
        },
    },
    madness: ModType {
        id: 7,
        prefix_name: "disturbing",
        roll_range: 10..=20,
        attune: Some(&attune::PENETRATION),
        show_tooltip: |this, ui, roll| { tooltip!("Overcome %roll %range %madness Resistance"); },
        register: |hooks, _item, _equip, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.penetration.madness += roll as f32);
        },
    },
    void: ModType {
        id: 8,
        prefix_name: "silent", // TODO can do better hollow, vast eternal, perpetual
        roll_range: 10..=20,
        attune: Some(&attune::PENETRATION),
        show_tooltip: |this, ui, roll| { tooltip!("Overcome %roll %range %void Resistance"); },
        register: |hooks, _item, _equip, roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.penetration.void += roll as f32);
        },
    },
};

pub static CULLING: ModType = ModType {
    id: 9,
    prefix_name: "culling",
    roll_range: 5..=10,
    attune: None,
    show_tooltip: |this, ui, roll| { tooltip!("Kill enemies under %roll% %range of their max HP"); },
    register: |hooks, _item, _equip, roll| {
        hooks.on_post_hit(move |attack, _skill, _user, _target, _hit| attack.cull_threshhold = (roll as f32).at_least(attack.cull_threshhold));
    },
};

pub static LIFESTEAL: ModType = ModType {
    id: 10,
    prefix_name: "vampiric",
    roll_range: 4..=8,
    attune: None,
    show_tooltip: |this, ui, roll| { tooltip!("Steal %roll% %range of Damage dealt as Life"); },
    register: |hooks, _item, _equip, roll| {
        hooks.on_post_hit(move |attack, _skill, _user, _target, _hit| attack.life_steal += (roll as f32) / 100.);
    },
};
pub static SHIELDSTEAL: ModType = ModType {
    id: 11,
    prefix_name: "leeching",
    roll_range: 5..=15,
    attune: None,
    show_tooltip: |this, ui, roll| { tooltip!("Steal %roll% %range of Damage dealt as Shield"); },
    register: |hooks, _item, _equip, roll| {
        hooks.on_post_hit(move |attack, _skill, _user, _target, _hit| attack.shield_steal += (roll as f32) / 100.);
    },
};

// TODO more details: primary damage type, all debuffs
pub static DEBUFF_OFF_ST: ModType = ModType {
    id: 12,
    prefix_name: "afflicting",
    roll_range: 0..=0,
    attune: None,
    show_tooltip: |_this, ui, _roll| { tooltip!("Every 2nd attack inflicts the offensive Debuff of the primary damage type"); },
    register: |hooks, _item, _equip, _roll| {
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
    attune: None,
    show_tooltip: |_this, ui, _roll| { tooltip!("Every 3rd attack inflicts the offensive Debuff of the primary damage type"); },
    register: |hooks, _item, _equip, _roll| {
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
    attune: None,
    show_tooltip: |_this, ui, _roll| { tooltip!("Every 2nd attack gives the enemy the utility debuff of the primary damage type"); },
    register: |hooks, _item, _equip, _roll| {
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
    attune: None,
    show_tooltip: |_this, ui, _roll| { tooltip!("Every 3rd attack gives the enemy the utility debuff of the primary damage type"); },
    register: |hooks, _item, _equip, _roll| {
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
pub static ATK_READY: ModType = ModType {
    id: 16,
    prefix_name: "prepared",
    roll_range: 0..=0,
    attune: None,
    show_tooltip: |_this, ui, _roll| { tooltip!("Start Combat with the linked items skill ready"); },
    register: |hooks, item, equip, _roll| {
        if let Some(linked_id) = equip.get_linked_item(item).upgrade().map(|i| i.id) {
            hooks.on_combat_start(move |effects, _user| effects.ready_skills.push(linked_id));
        }
    },
};
pub static MULTISTRIKE_ST: ModType = ModType {
    id: 17,
    prefix_name: "dueling",
    roll_range: 0..=0,
    attune: None,
    show_tooltip: |_this, ui, _roll| { tooltip!("When your offhand is empty, you main hand strikes one more time"); },
    register: |hooks, _item, equip, _roll| {
        let (off_hand, two_handed) = equip.get_item(ItemSlot::Weapon(1));
        if !two_handed && off_hand.upgrade().is_none() {
            if let Some(main_hand_id) = equip.get_item(ItemSlot::Weapon(0)).0.upgrade().map(|i| i.id) {
                hooks.on_pre_attack(move |attack, skill, _user, _targets| {
                    if let SkillSource::Item { id, .. } = skill.source {
                        if id == main_hand_id {
                            attack.hits += 1;
                        }
                    }
                });
            }
        }
    },
};
pub static MULTISTRIKE_AOE: ModType = ModType {
    id: 18,
    prefix_name: "focussed",
    roll_range: 0..=0,
    attune: None,
    show_tooltip: |_this, ui, _roll| { tooltip!("When there is only 1 target, you strike one more time"); },
    register: |hooks, _item, _equip, _roll| {
        hooks.on_pre_attack(move |attack, _skill, _user, targets| {
            if targets.len() == 1 {
                    attack.hits += 1;
            }
        });
    },
};

pub static PEN_CONVERSION: Elemental<ModType> = Elemental {
    bleed: ModType {
        id: 19,
        prefix_name: "omni-sharp",
        roll_range: 0..=0,
        attune: Some(&attune::PEN_CONVERSION),
        show_tooltip: |_this, ui, _roll| { tooltip!("%bleed penetration counts against all resistances"); },
        register: |hooks, _item, _equip, _roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.pen_conversion.bleed = true);
        },
    },
    fracture: ModType {
        id: 20,
        prefix_name: "omni-grinding",
        roll_range: 0..=0,
        attune: Some(&attune::PEN_CONVERSION),
        show_tooltip: |_this, ui, _roll| { tooltip!("%fracture penetration counts against all resistances"); },
        register: |hooks, _item, _equip, _roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.pen_conversion.fracture = true);
        },
    },
    madness: ModType {
        id: 21,
        prefix_name: "omni-disturbing",
        roll_range: 0..=0,
        attune: Some(&attune::PEN_CONVERSION),
        show_tooltip: |_this, ui, _roll| { tooltip!("%madness penetration counts against all resistances"); },
        register: |hooks, _item, _equip, _roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.pen_conversion.madness = true);
        },
    },
    void: ModType {
        id: 22,
        prefix_name: "omni-silent", // TODO can do better hollow, vast eternal, perpetual
        roll_range: 0..=0,
        attune: Some(&attune::PEN_CONVERSION),
        show_tooltip: |_this, ui, _roll| { tooltip!("%void penetration counts against all resistances"); },
        register: |hooks, _item, _equip, _roll| {
            hooks.on_pre_hit(move |attack, _skill, _user, _target| attack.pen_conversion.void = true);
        },
    },
};

pub static LIGHT: ModType = ModType {
    id: 23,
    prefix_name: "light",
    roll_range: 0..=0,
    attune: None,
    show_tooltip: |_this, ui, _roll| { tooltip!("This Axe can be worn in the Shield slot"); },
    register: |_hooks, _item, _equip, _roll| {}
};