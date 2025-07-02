use std::fmt::Debug;

use crate::{combat::{combatant::{CharStats, Combatant}, skill::{attack::{AttackPostHit, AttackPreHit, ResponsePostHit, ResponsePreHit}, defend::Defend, hit::Hit, skill::Skill}}, prelude::*};

#[derive(Default)]
pub struct CombatHooks {
    pre_hit: Vec<Box<dyn PreHitHook>>,
    post_hit: Vec<Box<dyn PostHitHook>>,
    resp_pre_atk: Vec<Box<dyn PreAttackHook>>,
    resp_post_atk: Vec<Box<dyn PostAttackHook>>,
    defend: Vec<Box<dyn DefHook>>,
    char: Vec<Box<dyn CharHook>>,
    combat_start: Vec<Box<dyn CombatStartHook>>,
    // evtl die buff only hooks <- think about how buffs play into this
}

impl CombatHooks {
    pub fn on_pre_hit(&mut self, hook: impl PreHitHook) {
        self.pre_hit.push(Box::new(hook));
    }
    pub fn on_post_hit(&mut self, hook: impl PostHitHook) {
        self.post_hit.push(Box::new(hook));
    }
    pub fn on_resp_pre_atk(&mut self, hook: impl PreAttackHook) {
        self.resp_pre_atk.push(Box::new(hook));
    }
    pub fn on_resp_post_atk(&mut self, hook: impl PostAttackHook) {
        self.resp_post_atk.push(Box::new(hook));
    }
    pub fn on_defend(&mut self, hook: impl DefHook) {
        self.defend.push(Box::new(hook));
    }
    pub fn on_char(&mut self, hook: impl CharHook) {
        self.char.push(Box::new(hook));
    }
    pub fn on_combat_start(&mut self, hook: impl CombatStartHook) {
        self.combat_start.push(Box::new(hook));
    }

    pub fn pre_hit(&self, attack: &mut AttackPreHit, skill: &Skill, user: &Combatant, target: &Combatant) {
        self.pre_hit.iter().for_each(|hook| hook(attack, skill, user, target));
    }
    pub fn post_hit(&self, attack: &mut AttackPostHit, skill: &Skill, user: &Combatant, target: &Combatant, hit: &Hit) {
        self.post_hit.iter().for_each(|hook| hook(attack, skill, user, target, hit));
    }
    pub fn resp_pre_atk(&self, resp: &mut ResponsePreHit, skill: &Skill, user: &Combatant, attacker: &Combatant) {
        self.resp_pre_atk.iter().for_each(|hook| hook(resp, skill, user, attacker));
    }
    pub fn resp_post_atk(&self, resp: &mut ResponsePostHit, skill: &Skill, user: &Combatant, attacker: &Combatant, hit: &Hit) {
        self.resp_post_atk.iter().for_each(|hook| hook(resp, skill, user, attacker, hit));
    }
    pub fn defend(&self, def: &mut Defend, skill: &Skill, user: &Combatant) {
        self.defend.iter().for_each(|hook| hook(def, skill, user));
    }
    pub fn char(&self, char: &mut CharStats) {
        self.char.iter().for_each(|hook| hook(char));
    }
    pub fn combat_start(&self) {
        self.combat_start.iter().for_each(|hook| hook());
    }
}
impl Debug for CombatHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CombatHooks")
            .field("pre_hit", &self.pre_hit.len())
            .field("post_hit", &self.post_hit.len())
            .field("resp_pre_atk", &self.resp_pre_atk.len())
            .field("resp_post_atk", &self.resp_post_atk.len())
            .field("defend", &self.defend.len())
            .field("char", &self.char.len())
            .field("combat_start", &self.combat_start.len())
            .finish()
    }
}

pub trait PreHitHook: Fn(&mut AttackPreHit, &Skill, &Combatant, &Combatant) + 'static {}
pub trait PostHitHook: Fn(&mut AttackPostHit, &Skill, &Combatant, &Combatant, &Hit) + 'static {}
pub trait PreAttackHook: Fn(&mut ResponsePreHit, &Skill, &Combatant, &Combatant) + 'static {}
pub trait PostAttackHook: Fn(&mut ResponsePostHit, &Skill, &Combatant, &Combatant, &Hit) + 'static {}
pub trait DefHook: Fn(&mut Defend, &Skill, &Combatant) + 'static {}
pub trait CharHook: Fn(&mut CharStats) + 'static {}
pub trait CombatStartHook: Fn() + 'static {}

impl<T: Fn(&mut AttackPreHit, &Skill, &Combatant, &Combatant) + 'static> PreHitHook for T {}
impl<T: Fn(&mut AttackPostHit, &Skill, &Combatant, &Combatant, &Hit) + 'static> PostHitHook for T {}
impl<T: Fn(&mut ResponsePreHit, &Skill, &Combatant, &Combatant) + 'static> PreAttackHook for T {}
impl<T: Fn(&mut ResponsePostHit, &Skill, &Combatant, &Combatant, &Hit) + 'static> PostAttackHook for T {}
impl<T: Fn(&mut Defend, &Skill, &Combatant) + 'static> DefHook for T {}
impl<T: Fn(&mut CharStats) + 'static> CharHook for T {}
impl<T: Fn() + Debug + 'static> CombatStartHook for T {}