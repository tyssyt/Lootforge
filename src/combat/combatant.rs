use std::mem;

use crate::combat::buff::Buffs;
use crate::combat::hooks::CombatHooks;
use crate::equipment::equipment::FighterEquip;
use crate::item::Item;
use crate::prelude::*;

use crate::{
    elemental::Elemental, item::ItemRef, mods::*,
    panels::animation::Animation,
};

use super::skill::attack::{AttackPreHit, ResponsePostHit, ResponsePreHit};
use super::skill::hit::Hit;
use super::skill::skill::{SkillSource, SkillStats};
use super::skill::targeting::Targeting;
use super::{
    enemy::EnemyKind,
    skill::skill::{Skill, SkillKind},
};

#[derive(Debug, Clone, Copy, EnumIs)]
pub enum CombatantKind {
    Fighter,
    // Ranger,
    // Mage,
    Enemy(u8, EnemyKind),
}

#[derive(Debug)]
pub struct Combatant {
    pub kind: CombatantKind,

    pub health: f32,
    pub shield: f32,
    pub buffs: Buffs,
    pub skills: Vec<Skill>,
    pub hooks: CombatHooks,
}

impl Combatant {
    pub fn transfer(&mut self) {
        self.shield = 0.;
        self.buffs = Buffs::default();
        self.skills.iter_mut().for_each(|s| s.transfer());
    }

    pub fn fighter<'a>(equip: &FighterEquip) -> Self {
        let item_none: Weak<Item> = Weak::new();
        let skills = [
            (&equip.weapons[0], &equip.common.rings[0]),
            (&equip.weapons[1], &equip.common.rings[1]),
            (&equip.shield, &equip.common.rings[2]),
            (&equip.common.helmet, &item_none),
        ]
        .into_iter()
        .map(|(i, r)| (i.upgrade(), r.upgrade()))
        .filter(|(i, _)| i.is_some())
        .map(|(i, r)| (i.unwrap(), r))
        .filter_map(|(i, r)| Skill::from_item(i, r.and_then(|r| r.targeting)))
        .collect();

        Self::explorer(CombatantKind::Fighter, skills, equip.iter())
    }

    fn explorer<'a>(
        kind: CombatantKind,
        skills: Vec<Skill>,
        equip: impl Iterator<Item = &'a ItemRef>,
    ) -> Self {
        let mut hooks = CombatHooks::default();
        equip.filter_map(|i| i.upgrade())
            .filter(|i| SkillKind::from_item_type(i.item_type).is_none())
            // can't do flatmap because rust -.-
            .for_each(|i| i.mods.iter().for_each(|m| m.register(&mut hooks)));

        let mut explorer = Self {
            kind,
            health: 0.,
            shield: 0.,
            buffs: Buffs::default(),
            skills,
            hooks,
        };
        explorer.health = explorer.stats().max_health;
        explorer
    }

    // ranger
    // mage

    pub fn enemy<R: Rng>(rng: &mut R, kind: EnemyKind, i: u8, depth: u16) -> Self {
        let mut hooks = CombatHooks::default();
        atk_mod::ADDED_DMG.choose(rng).roll(rng).register(&mut hooks); // TODO register this in the skill?
        hooks.on_pre_hit(
            move |attack: &mut AttackPreHit, _skill: &Skill, _user: &Combatant, _target: &Combatant| attack.penetration = attack.penetration + depth as f32
        );
        hooks.on_char(
            move |char: &mut CharStats| char.resistances = char.resistances + 2. * (depth as f32)
        );

        let mut enemy = Self {
            kind: CombatantKind::Enemy(i, kind),
            health: 0.,
            shield: 0.,
            buffs: Buffs::default(),
            skills: kind.skills(),
            hooks,
        };
        enemy.health = enemy.stats().max_health;
        enemy
    }

    pub fn alive(&self) -> bool {
        self.health > 0.
    }

    // TODO cache Stats
    pub fn stats(&self) -> CharStats {
        let mut char = CharStats::default();
        self.hooks.char(&mut char);
        self.buffs.apply_to_char(&mut char);
        char
    }

    pub fn tick<'a>(
        &mut self,
        tick: u64,
        mut allies: Vec<&'a mut Combatant>,
        mut enemies: Vec<&'a mut Combatant>,
    ) -> Option<SkillStats> {
        Buffs::tick(self);

        let stats = self.stats();

        if tick % (stats.tick_rate as u64) != 0 {
            return None;
        }

        self.skills.iter_mut().for_each(|s| s.tick());

        if self.skills.iter().any(|s| s.ready()) {
            self.trigger_skill(&mut allies, &mut enemies, true)
        } else {
            None
        }
    }

    pub fn trigger_skill<'a, 'b>(
        &mut self,
        allies: &'a mut Vec<&'b mut Combatant>,
        enemies: &'a mut Vec<&'b mut Combatant>,
        reset_cooldown: bool,
    ) -> Option<SkillStats> {
        let mut skills = mem::take(&mut self.skills);

        let ready = skills.iter_mut()
            .filter(|s| s.ready())
            .filter(|s| s.targeting != Targeting::OnAttack)
            .next();

        let stats = ready.map(|s| s.trigger(self, allies, enemies, reset_cooldown));

        self.skills = skills; // TODO why has rust no defer???
        stats
    }

    pub fn trigger_skill_against_attack(&mut self, attacker: &mut Combatant, reset_cooldown: bool) -> Option<(ResponsePreHit, SkillStats)> {
        let mut skills = mem::take(&mut self.skills);

        let ready = skills.iter_mut()
            .filter(|s| s.ready())
            .filter(|s| s.targeting == Targeting::OnAttack)
            .next();

        let resp = ready.map(|skill| {
            let stats = skill.trigger_against_target(self, attacker, reset_cooldown);

            let mut resp = ResponsePreHit::default();
            skill.hooks.resp_pre_atk(&mut resp, skill, self, attacker);
            self.hooks.resp_pre_atk(&mut resp, skill, self, attacker);
            self.buffs.apply_pre_atk(&mut resp, skill, self, attacker);

            (resp, stats)
        });

        self.skills = skills; // TODO why has rust no defer???
        resp
    }
    pub fn trigger_post_attack(&mut self, id: &SkillSource, attacker: &Self, hit: &Hit) -> ResponsePostHit {
        let mut skills = mem::take(&mut self.skills);

        let skill = skills.iter_mut()
            .find(|s| &s.source == id)
            .unwrap();
        
        let mut resp = ResponsePostHit::default();
        skill.hooks.resp_post_atk(&mut resp, skill, self, attacker, hit);
        self.hooks.resp_post_atk(&mut resp, skill, self, attacker, hit);
        self.buffs.apply_post_atk(&mut resp, skill, self, attacker, hit);
        
        self.skills = skills; // TODO why has rust no defer???
        resp
    }

    pub fn trigger_attack_against_target(&mut self, target: &mut Combatant, reset_cooldown: bool) -> Option<SkillStats> {
        let mut skills = mem::take(&mut self.skills);

        let skill = skills.iter_mut().filter(|s| s.kind.is_attack()).next();

        let stats = skill.map(|s| s.trigger_against_target(self, target, reset_cooldown));

        self.skills = skills; // TODO why has rust no defer???
        stats
    }

    pub fn idle_animation(&self) -> Animation {
        match &self.kind {
            CombatantKind::Fighter => Animation::FighterAttack,
            CombatantKind::Enemy(_, enemy_kind) => enemy_kind.idle_animation(),
        }
    }

    pub fn damage(&mut self, amount: f32) {  
        let shield_dmg = (amount * 0.8).at_most(self.shield);
        let health_dmg = (amount - shield_dmg).at_most(self.health);

        self.health -= health_dmg;
        self.shield -= shield_dmg;
    }
    pub fn heal(&mut self, amount: f32) -> f32 {
        let char = self.stats();
        let total_heal = amount * char.heal_power;
        let healed = total_heal.at_most(char.max_health - self.health);
        self.health += healed;
        healed
    }
    pub fn shield(&mut self, amount: f32) -> f32 {
        let total_shield = amount * self.stats().shield_power;
        self.shield += total_shield;
        total_shield
    }
}

pub struct CharStats {
    pub max_health: f32,
    pub resistances: Elemental<f32>,

    pub heal_power: f32,
    pub shield_power: f32,
    pub cdr: u16,
    pub tick_rate: u8,
}

impl Default for CharStats {
    fn default() -> Self {
        Self {
            max_health: 1000.,
            resistances: Elemental::from(0.),
            heal_power: 1.,
            shield_power: 1.,
            cdr: 0,
            tick_rate: 1,
        }
    }
}
