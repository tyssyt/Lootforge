use crate::prelude::*;
use crate::combat::buff::Buff;
use super::ModType;

pub static SHIELD: ModType = ModType {
    id: 30,
    prefix_name: "shielding",
    roll_range: 20..=50,
    show_tooltip: |this, ui, roll| { tooltip!("Gain %roll %range shield"); },
    register: |hooks, roll| {
        hooks.on_defend(move |def, _skill, _user| def.shield += roll as f32);
    },    
};

pub static HEAL: ModType = ModType {
    id: 31,
    prefix_name: "healing",
    roll_range: 15..=30,
    show_tooltip: |this, ui, roll| { tooltip!("Heal %roll %range Hitpoints"); },
    register: |hooks, roll| {
        hooks.on_defend(move |def, _skill, _user| def.heal += roll as f32,);
    },    
};

pub static BLOCK: ModType = ModType {
    id: 32,
    prefix_name: "blocking",
    roll_range: 0..=0,
    show_tooltip: |_this, ui, _roll| { tooltip!("Completely Negate the attack"); },
    register: |hooks, _roll| {
        hooks.on_resp_pre_atk(move |resp, _user, _skill, _attacker| resp.block = true);
    },
};

// TODO we need a "more details" in tooltip. Either a link to book, or another sub tooltip that appears on hover
// explain that the counter will use the first (leftmose) weapon and only target the attacker, even if aoe
pub static COUNTER: ModType = ModType {
    id: 33,
    prefix_name: "counter",
    roll_range: 0..=0,
    show_tooltip: |_this, ui, _roll| { tooltip!("Trigger a counter attack against the attacker"); },
    register: |hooks, _roll| {
        hooks.on_resp_post_atk(move |resp, _skill, _user, _attacker, _hit| resp.counter = true);
    },
};

// TODO also extended tooltip explaining that the buff stays the entire combat encounter, and stacks
pub static ATTUNE: ModType = ModType {
    id: 34,
    prefix_name: "attuning",
    roll_range: 3..=6,
    show_tooltip: |this, ui, roll| { tooltip!("Grants a stackable Buff that gives %roll %range resistance against the attacks primary damage type"); },
    register: |hooks, roll| {
        hooks.on_resp_post_atk(move |resp, _skill, _user, _attacker, hit| resp.buffs.push(Buff::attuned(roll as f32, hit.post_res_dmg.max_idx())));
    },
};
pub static REVERB: ModType = ModType {
    id: 35,
    prefix_name: "reverberant",
    roll_range: 0..=0,
    show_tooltip: |_this, ui, _roll| { tooltip!("Grants a Buff that will add the taken Damage to your next attack"); },
    register: |hooks, _roll| {
        hooks.on_resp_post_atk(move |resp, _skill, _user, _attacker, hit| resp.buffs.push(Buff::reverb(hit.pre_res_dmg)));
    },
};