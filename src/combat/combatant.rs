use std::mem;

use crate::combat::buff::Buffs;
use crate::combat::hooks::CombatHooks;
use crate::elemental::Element;
use crate::equipment::equipment::{Equip, EquipEnum, FighterEquip};
use crate::prelude::*;

use crate::{elemental::Elemental, panels::animation::Animation};

use super::skill::hit::{Hit, PreHit, ResponsePostHit, ResponsePreHit};
use super::skill::skill::{SkillSource, SkillStats};
use super::skill::targeting::Targeting;
use super::{
    enemy::EnemyKind,
    skill::skill::{Skill, SkillKind},
};

#[apply(Enum)]
#[derive(Copy)]
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
    pub wounds: f32,
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

    pub fn fighter(equip: &FighterEquip) -> Self {
        let equip_enum = equip.clone().into();
        let skills = [
            &equip.weapons[0],
            &equip.weapons[1],
            &equip.shield,
            &equip.common.helmet,
        ]
        .into_iter()
        .filter_map(|i| i.upgrade())
        .filter_map(|i| Skill::from_item(i, &equip_enum))
        .collect();

        Self::explorer(CombatantKind::Fighter, skills, equip_enum)
    }

    fn explorer(kind: CombatantKind, skills: Vec<Skill>, equip: EquipEnum) -> Self {
        let mut hooks = CombatHooks::default();
        equip
            .iter()
            .filter_map(|i| i.upgrade())
            .filter(|i| SkillKind::from_item_type(i.item_type).is_none())
            // can't do flatmap because rust -.-
            .for_each(|i| {
                i.mods.iter().for_each(|m| m.register(&mut hooks, &i, &equip))
            });

        let mut explorer = Self {
            kind,
            health: 0.,
            wounds: 0.,
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

    pub fn enemy(kind: EnemyKind, i: u8, depth: u16, rng: &mut impl Rng) -> Self {
        let mut hooks = CombatHooks::default();
        hooks.on_pre_hit(move |attack: &mut PreHit, _skill: &Skill, _user: &Combatant, _target: &Combatant| {
            attack.penetration = attack.penetration + depth as f32;
        });
        hooks.on_char(move |char: &mut CharStats| {
            char.resistances = char.resistances + (depth as f32) / 2.;
        });
        
        let damage_type = *Element::VARIANTS.pick(rng);
        let mut enemy = Self {
            kind: CombatantKind::Enemy(i, kind),
            health: 0.,
            wounds: 0.,
            shield: 0.,
            buffs: Buffs::default(),
            skills: kind.etype().skills(damage_type),
            hooks,
        };
        enemy.health = enemy.stats().max_health;

        let initial_skill_delay = rng.random_range(0..=20);
        enemy.skills.iter_mut().for_each(|s| s.cd += initial_skill_delay);

        enemy
    }
}
impl Combatant {
    pub fn combat_start(&mut self) {
        // add delay to off hand
        self.skills.iter_mut()
            .filter(|s| s.kind() == SkillKind::Attack)
            .nth(1)
            .map(|s| s.cd += 10);

        let mut effects = CombatStartEffects::default();
        self.hooks.combat_start(&mut effects, self);
        self.skills.iter().for_each(|s| s.hooks.combat_start(&mut effects, self));

        let stats = self.stats();
        self.shield(effects.shield_from_max_health * stats.max_health);
        for ready_skill in effects.ready_skills {
            self.find_skill_mut(ready_skill).map(|s| s.cd = 0);
        }
    }

    pub fn alive(&self) -> bool {
        self.health > 0.
    }

    pub fn find_skill(&self, item_id: usize) -> Option<&Skill> {
        self.skills.iter().find(|s| match s.source {
            SkillSource::Item { id, .. } => item_id == id,
            SkillSource::Enemy { .. } => false,
        })
    }
    pub fn find_skill_mut(&mut self, item_id: usize) -> Option<&mut Skill> {
        self.skills.iter_mut().find(|s| match s.source {
            SkillSource::Item { id, .. } => item_id == id,
            SkillSource::Enemy { .. } => false,
        })
    }

    // TODO cache Stats
    pub fn stats(&self) -> CharStats {
        let mut char = self.kind.stats();
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

        let ready = skills
            .iter_mut()
            .filter(|s| s.ready())
            .filter(|s| s.targeting != Targeting::OnAttack)
            .next();

        let stats = ready.map(|s| s.trigger(self, allies, enemies, reset_cooldown));

        self.skills = skills; // TODO why has rust no defer???
        stats
    }

    pub fn trigger_skill_against_attack(&mut self, attacker: &mut Combatant, reset_cooldown: bool) -> Option<(ResponsePreHit, SkillStats)> {
        let mut skills = mem::take(&mut self.skills);

        let ready = skills
            .iter_mut()
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

        let skill = skills.iter_mut().find(|s| &s.source == id).unwrap();

        let mut resp = ResponsePostHit::default();
        skill.hooks.resp_post_atk(&mut resp, skill, self, attacker, hit);
        self.hooks.resp_post_atk(&mut resp, skill, self, attacker, hit);
        self.buffs.apply_post_atk(&mut resp, skill, self, attacker, hit);

        self.skills = skills; // TODO why has rust no defer???
        resp
    }

    pub fn trigger_attack_against_target(&mut self, target: &mut Combatant, reset_cooldown: bool) -> Option<SkillStats> {
        let mut skills = mem::take(&mut self.skills);
        let skill = skills.iter_mut().filter(|s| s.kind().is_attack()).next();
        let stats = skill.map(|s| s.trigger_against_target(self, target, reset_cooldown));
        self.skills = skills; // TODO why has rust no defer???
        stats
    }

    pub fn idle_animation(&self) -> Animation {
        match &self.kind {
            CombatantKind::Fighter => Animation::FighterIdle,
            CombatantKind::Enemy(_, enemy_kind) => enemy_kind.idle_animation(),
        }
    }
    pub fn attack_animation(&self) -> Animation {
        match &self.kind {
            CombatantKind::Fighter => Animation::FighterAttack,
            CombatantKind::Enemy(_, enemy_kind) => enemy_kind.attack_animation(),
        }
    }

    pub fn damage(&mut self, amount: f32) {
        let char = self.stats();
        let shield_dmg = (amount * 0.75).at_most(self.shield);
        let health_dmg = amount - shield_dmg;

        self.health -= health_dmg.at_most(self.health);
        self.wounds += (health_dmg / 4.).at_most(char.max_health - self.wounds);
        self.shield -= shield_dmg;
    }
    pub fn heal(&mut self, amount: f32) -> f32 {
        let char = self.stats();
        let total_heal = amount * char.heal_power;
        let healed = total_heal.at_most(char.max_health - self.wounds - self.health);
        self.health += healed;
        healed
    }
    pub fn shield(&mut self, amount: f32) -> f32 {
        let char = self.stats();
        let total_shield = amount * char.shield_power;
        self.shield += total_shield.at_most(char.max_health - self.shield);
        total_shield
    }
}
impl CombatantKind {
    fn stats(&self) -> CharStats {
        match self {
            CombatantKind::Fighter => CharStats {
                max_health: 500.,
                resistances: Default::default(),
                heal_power: 1.0,
                shield_power: 1.0,
                cdr: 0,
                tick_rate: 1,
            },
            CombatantKind::Enemy(_, enemy_kind) => enemy_kind.etype().stats(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CharStats {
    pub max_health: f32,
    pub resistances: Elemental<f32>,

    pub heal_power: f32,
    pub shield_power: f32,
    pub cdr: u16,
    pub tick_rate: u8,
}

#[apply(Default)]
pub struct CombatStartEffects {
    pub shield_from_max_health: f32,
    pub ready_skills: Vec<usize>,
}
