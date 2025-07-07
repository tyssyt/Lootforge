use crate::combat::buff::{Buff, Debuff};
use crate::combat::skill::attack::AttackStats;
use crate::combat::skill::skill::SkillStats;
use crate::elemental::Element;
use crate::prelude::*;
use crate::combat::combatant::CombatantKind;

use crate::{combat::{combatant::Combatant}, elemental::Elemental};

use super::skill::Skill;

#[apply(Default)]
pub struct Hit {
    pub pre_res_dmg: Elemental<f32>,
    pub post_res_dmg: Elemental<f32>,
    pub penetration: Elemental<f32>,
    pub ignore_res: Elemental<bool>,
}

#[apply(Default)]
pub struct PreHit {
    pub damage: Elemental<f32>,
    #[default(Elemental::from(1.))]
    pub damage_mult: Elemental<f32>,
    pub penetration: Elemental<f32>,
    pub pen_conversion: Elemental<bool>,
    pub ignore_res: Elemental<bool>,
    pub debuffs: Vec<Debuff>,
}

#[apply(Default)]
pub struct PostHit {
    pub debuffs: Vec<Debuff>,
    pub cull_threshhold: f32,
    pub life_steal: f32,
    pub shield_steal: f32,
    pub self_hit: bool,
}

#[apply(Default)]
pub struct ResponsePreHit {
    pub block: bool,
    pub debuffs: Vec<Debuff>,
}
#[apply(Default)]
pub struct ResponsePostHit {
    pub counter: bool,
    pub buffs: Vec<Buff>,
    pub debuffs: Vec<Debuff>,
}

pub fn hit(skill: &Skill, user: &mut Combatant, target: &mut Combatant) -> HitStats {
    let (hit, response) = pre_hit(skill, user, target);
    if let Some(hit) = hit {
        post_hit(skill, user, target, hit, response)
    } else {
        HitStats { target: target.kind, hit: Hit::default(), responses: response.into_iter().collect() }
    }
}


fn pre_hit(skill: &Skill, user: &mut Combatant, target: &mut Combatant) -> (Option<Hit>, Option<SkillStats>) {
    let mut attack = PreHit::default();
    skill.hooks.pre_hit(&mut attack, skill, user, target);
    user.hooks.pre_hit(&mut attack, skill, user, target);
    user.buffs.apply_pre_hit(&mut attack, skill, user, target);
    target.buffs.apply_pre_getting_hit(&mut attack);

    let (response, response_stats) = target.trigger_skill_against_attack(user, true).split();
    if let Some(response) = response {
        response.debuffs.into_iter().for_each(|b| user.buffs.add(b));

        if response.block {
            // make sure all other effects that happen later are exclusive with block
            return (None, response_stats);
        }
    }

    // trigger attack pre hit effects
    attack.debuffs.into_iter().for_each(|b| target.buffs.add(b));

    // hit
    let pre_res_dmg = attack.damage * attack.damage_mult;

    let target_stats = target.stats();
    let penetration = penetration(attack.penetration, attack.pen_conversion);
    let effective_res = target_stats.resistances - penetration;
    let mut post_res_dmg = mitigation(pre_res_dmg, effective_res);
    post_res_dmg.assign_cond(pre_res_dmg, attack.ignore_res);

    target.damage(post_res_dmg.sum());
    let hit = Hit { pre_res_dmg, post_res_dmg, penetration, ignore_res: attack.ignore_res };

    ( Some(hit), response_stats )
}

fn post_hit(skill: &Skill, user: &mut Combatant, target: &mut Combatant, hit: Hit, response: Option<SkillStats>) -> HitStats {
    let mut attack = PostHit::default();
    skill.hooks.post_hit(&mut attack, skill, user, target, &hit);
    user.hooks.post_hit(&mut attack, skill, user, target, &hit);
    user.buffs.apply_post_hit(&mut attack, skill, user, target, &hit);
    target.buffs.apply_post_getting_hit(&mut attack);

    let mut responses: Vec<SkillStats> = Vec::new();

    let user_stats = user.stats();
    let target_stats = target.stats();

    if attack.self_hit {
        let effective_res = user_stats.resistances - hit.penetration;
        let mut post_res_dmg = mitigation(hit.pre_res_dmg, effective_res);
        post_res_dmg.assign_cond(hit.pre_res_dmg, hit.ignore_res);

        user.damage(post_res_dmg.sum());
        // TODO maybe introduce a shorter way to do this?
        let self_hit = Hit { pre_res_dmg: hit.pre_res_dmg, post_res_dmg, penetration: hit.penetration, ignore_res: hit.ignore_res };
        let self_hit_stats = HitStats { target: user.kind, hit: self_hit, responses: Vec::new() };
        let self_attack_stats = AttackStats { attacker: user.kind, hits: vec![self_hit_stats] };
        responses.push( SkillStats::Attack( skill.source.clone(), self_attack_stats) );
    }

    let dmg = hit.post_res_dmg.sum();
    if attack.life_steal > 0. {
        user.heal(dmg * attack.life_steal);
    }
    if attack.shield_steal > 0. {
        user.shield(dmg * attack.shield_steal);
    }

    attack.debuffs.into_iter().for_each(|b| target.buffs.add(b));

    let percent_health = target.health / target_stats.max_health;
    if percent_health < attack.cull_threshhold / 100. {
        target.health = 0.;
    }

    // TODO somewhere (prolly not here) do taunt calculation

    if let Some(stats) = &response {
        let resp = target.trigger_post_attack(stats.source(), user, &hit);

        if resp.counter {
            let counter = target.trigger_attack_against_target(user, false);
            counter.map(|c| responses.push(c));
        }

        resp.buffs.into_iter().for_each(|b| target.buffs.add(b));
        resp.debuffs.into_iter().for_each(|b| user.buffs.add(b));

    }

    // multiattack happens after this

    response.map(|r| responses.insert(0, r));
    HitStats { target: target.kind, hit, responses }
}

pub fn effective_health<F: Copy + ops::Mul<Output=F> + ops::Add<Output=F> + NumExt + From<f32>>(health: F, res: F) -> F {
    // TODO check if this is correct
    health * ((res.at_least(F::from(-99.)) * F::from(0.01)) + F::from(1.))
}

pub fn mitigation(dmg: Elemental<f32>, res: Elemental<f32>) -> Elemental<f32> {
    // TODO check if this is correct
    dmg / ((res.at_least(-99.) * 0.01) + 1.)
}

pub fn penetration(penetration: Elemental<f32>, pen_conversion: Elemental<bool>) -> Elemental<f32> {
    let mut total_pen = penetration.clone();
    Element::iter().for_each(|element| {
        if *pen_conversion.get(element) {
            total_pen = total_pen + *penetration.get(element);
            *total_pen.get_mut(element) -= penetration.get(element);
        }
    });
    total_pen
}

#[derive(Debug, Clone)]
pub struct HitStats {
    pub target: CombatantKind,
    pub hit: Hit,
    pub responses: Vec<SkillStats>,
}