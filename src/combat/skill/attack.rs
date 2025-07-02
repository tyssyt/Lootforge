use crate::prelude::*;

use crate::combat::skill::hit::{self, HitStats};
use crate::{combat::{buff::{Buff, Debuff}, combatant::{Combatant, CombatantKind}}, elemental::Elemental};
use super::skill::Skill;

#[derive(Debug)]
pub struct AttackPreHit {
    pub damage: Elemental<f32>,
    pub damage_mult: Elemental<f32>,
    pub penetration: Elemental<f32>,
    pub ignore_res: Elemental<bool>,
    pub debuffs: Vec<Debuff>,
}
impl Default for AttackPreHit {
    fn default() -> Self {
        Self {
            damage: Elemental::from(0.),
            damage_mult: Elemental::from(1.),
            penetration: Elemental::from(0.),
            ignore_res: Elemental::from(false),
            debuffs: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct AttackPostHit {
    pub debuffs: Vec<Debuff>,
    pub cull_threshhold: f32,
    pub life_steal: f32,
    pub shield_steal: f32,
    pub self_hit: bool,
}

#[derive(Default)]
pub struct ResponsePreHit {
    pub block: bool,
    pub debuffs: Vec<Debuff>,
}
#[derive(Default)]
pub struct ResponsePostHit {
    pub counter: bool,
    pub buffs: Vec<Buff>,
    pub debuffs: Vec<Debuff>,
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
    let hit_stats = hit::hit(skill, user, target);

    user.buffs.attacked();
    AttackStats { attacker: user.kind, hits: vec![hit_stats] }
}

pub fn attack_aoe(
    skill: &mut Skill,
    user: &mut Combatant,
    _allies: &mut Vec<&mut Combatant>,
    enemies: &mut Vec<&mut Combatant>,
) -> AttackStats {
    let hits = enemies.iter_mut()
        .map(|target| {
            hit::hit(skill, user, target)
        }).collect();
        
    user.buffs.attacked();
    AttackStats { attacker: user.kind, hits }
}

pub struct AttackStats {
    pub attacker: CombatantKind,
    pub hits: Vec<HitStats>,
}