use crate::prelude::*;
use crate::combat::{buff::Buff, combatant::{Combatant, CombatantKind}};

use super::skill::Skill;

pub fn defend(
    skill: &mut Skill,
    user: &mut Combatant,
) -> DefStats {
    let stats = def(skill, user);
    skill.uses += 1;
    stats
}

#[derive(Default)]
pub struct Defend {
    pub shield: f32,
    pub heal: f32,
    pub buffs: Vec<Buff>,
}

fn def(skill: &Skill, user: &mut Combatant) -> DefStats {
    let mut defend = Defend::default();
    skill.hooks.defend(&mut defend, skill, user);
    user.hooks.defend(&mut defend, &skill, user);
    user.buffs.apply_to_def(&mut defend, skill, user);

    defend.buffs.into_iter().for_each(|b| user.buffs.add(b));

    let shielded = user.shield(defend.shield);
    let healed = user.heal(defend.heal);

    DefStats { defender: user.kind, healed, shielded }
}

#[derive(Debug, Clone)]
pub struct DefStats {
    pub defender: CombatantKind,
    pub healed: f32,
    pub shielded: f32,
}