use crate::prelude::*;
use crate::combat::{
    combatant::{CharStats, CombatStartEffects, Combatant},
    skill::{
        attack::PreAttack, defend::Defend, hit::{Hit, PostHit, PreHit, ResponsePostHit, ResponsePreHit}, skill::Skill
    },
};

#[derive(Default)]
pub struct CombatHooks {
    pre_attack: Vec<Box<dyn PreAttackHook>>,
    pre_hit: Vec<Box<dyn PreHitHook>>,
    post_hit: Vec<Box<dyn PostHitHook>>,
    resp_pre_atk: Vec<Box<dyn PreAttackRespHook>>,
    resp_post_atk: Vec<Box<dyn PostAttackRespHook>>,
    defend: Vec<Box<dyn DefHook>>,
    char: Vec<Box<dyn CharHook>>,
    combat_start: Vec<Box<dyn CombatStartHook>>,
    // evtl die buff only hooks <- think about how buffs play into this
}

impl CombatHooks {
    pub fn on_pre_attack(&mut self, hook: impl PreAttackHook) {
        self.pre_attack.push(Box::new(hook));
    }
    pub fn on_pre_hit(&mut self, hook: impl PreHitHook) {
        self.pre_hit.push(Box::new(hook));
    }
    pub fn on_post_hit(&mut self, hook: impl PostHitHook) {
        self.post_hit.push(Box::new(hook));
    }
    pub fn on_resp_pre_atk(&mut self, hook: impl PreAttackRespHook) {
        self.resp_pre_atk.push(Box::new(hook));
    }
    pub fn on_resp_post_atk(&mut self, hook: impl PostAttackRespHook) {
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

    pub fn pre_attack(&self, attack: &mut PreAttack, skill: &Skill, user: &Combatant, targets: &Vec<&Combatant>) {
        self.pre_attack.iter().for_each(|hook| hook(attack, skill, user, targets));
    }
    pub fn pre_hit(&self, attack: &mut PreHit, skill: &Skill, user: &Combatant, target: &Combatant) {
        self.pre_hit.iter().for_each(|hook| hook(attack, skill, user, target));
    }
    pub fn post_hit(&self, attack: &mut PostHit, skill: &Skill, user: &Combatant, target: &Combatant, hit: &Hit) {
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
    pub fn combat_start(&self, effects: &mut CombatStartEffects, user: &Combatant) {
        self.combat_start.iter().for_each(|hook| hook(effects, user));
    }
}
impl std::fmt::Debug for CombatHooks {
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
pub trait PreAttackHook: Fn(&mut PreAttack, &Skill, &Combatant, &Vec<&Combatant>) + 'static {}
pub trait PreHitHook: Fn(&mut PreHit, &Skill, &Combatant, &Combatant) + 'static {}
pub trait PostHitHook: Fn(&mut PostHit, &Skill, &Combatant, &Combatant, &Hit) + 'static {}
pub trait PreAttackRespHook: Fn(&mut ResponsePreHit, &Skill, &Combatant, &Combatant) + 'static {}
pub trait PostAttackRespHook: Fn(&mut ResponsePostHit, &Skill, &Combatant, &Combatant, &Hit) + 'static {}
pub trait DefHook: Fn(&mut Defend, &Skill, &Combatant) + 'static {}
pub trait CharHook: Fn(&mut CharStats) + 'static {}
pub trait CombatStartHook: Fn(&mut CombatStartEffects, &Combatant) + 'static {}

impl<T: Fn(&mut PreAttack, &Skill, &Combatant, &Vec<&Combatant>) + 'static> PreAttackHook for T {}
impl<T: Fn(&mut PreHit, &Skill, &Combatant, &Combatant) + 'static> PreHitHook for T {}
impl<T: Fn(&mut PostHit, &Skill, &Combatant, &Combatant, &Hit) + 'static> PostHitHook for T {}
impl<T: Fn(&mut ResponsePreHit, &Skill, &Combatant, &Combatant) + 'static> PreAttackRespHook for T {}
impl<T: Fn(&mut ResponsePostHit, &Skill, &Combatant, &Combatant, &Hit) + 'static> PostAttackRespHook for T {}
impl<T: Fn(&mut Defend, &Skill, &Combatant) + 'static> DefHook for T {}
impl<T: Fn(&mut CharStats) + 'static> CharHook for T {}
impl<T: Fn(&mut CombatStartEffects, &Combatant) + 'static> CombatStartHook for T {}
