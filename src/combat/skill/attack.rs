use std::{iter};

use crate::prelude::*;
use crate::combat::skill::hit::{self, HitStats};
use crate::combat::combatant::{Combatant, CombatantKind};
use super::skill::Skill;

#[apply(Default)]
pub struct PreAttack {
    #[default(1)]
    pub hits: u8,
}

pub fn attack_single(
    skill: &mut Skill,
    user: &mut Combatant,
    _allies: &mut Vec<&mut Combatant>,
    enemies: &mut Vec<&mut Combatant>,
) -> AttackStats {
    let target = &mut skill.targeting.select_target(enemies);
    attack_target(skill, user, target)
}

pub fn attack_target(    
    skill: &mut Skill,
    user: &mut Combatant,
    target: &mut Combatant,
) -> AttackStats {
    let mut attack = PreAttack::default();
    let targets: Vec<&Combatant> = vec![target];
    skill.hooks.pre_attack(&mut attack, skill, user, &targets);
    user.hooks.pre_attack(&mut attack, skill, user, &targets);

    let hits = iter::repeat_n((), attack.hits as usize)
        .map(|_| {
            let hit_stats = hit::hit(skill, user, target);
            user.buffs.attacked();
            skill.uses += 1;
            hit_stats
        })
        .collect();

    AttackStats { attacker: user.kind, hits }
}

pub fn attack_aoe(
    skill: &mut Skill,
    user: &mut Combatant,
    _allies: &mut Vec<&mut Combatant>,
    enemies: &mut Vec<&mut Combatant>,
) -> AttackStats {
    let mut attack = PreAttack::default();
    let targets: Vec<&Combatant> = enemies.iter().map(|c| c as &Combatant).collect();
    skill.hooks.pre_attack(&mut attack, skill, user, &targets);
    user.hooks.pre_attack(&mut attack, skill, user, &targets);

    let hits = iter::repeat_n((), attack.hits as usize)
        .flat_map(|_| {
            let hits: Vec<_> = enemies.iter_mut()
                .map(|target| {
                    hit::hit(skill, user, target)
                }).collect();        
            user.buffs.attacked();
            skill.uses += 1;
            hits
        })
        .collect();

    AttackStats { attacker: user.kind, hits }
}

#[derive(Debug, Clone)]
pub struct AttackStats {
    pub attacker: CombatantKind,
    pub hits: Vec<HitStats>,
}