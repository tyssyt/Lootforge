use crate::{combat::{combatant::{CharStats, Combatant}, hooks::CombatHooks, skill::hit::PreHit}, elemental::{Element, Elemental}, panels::animation::Spritesheet, prelude::*};
use EnemyType::*;
use EnemyKind::*;

use crate::{
    combat::skill::skill::{Skill, SkillKind},
    panels::animation::Animation,
};

#[apply(UnitEnum)]
pub enum EnemyType {
    Small,
    Medium,
    Tank,
    Dps,
}
#[apply(UnitEnum)]
pub enum EnemyKind {
    Bat,
    BigWorn,
    Minotaur,
    Orc,
    Skeleton,
    Spider,
    BigSpider,
}

impl From<EnemyKind> for EnemyType {
    fn from(value: EnemyKind) -> Self {
        match value {
            Bat | Spider => Small,
            Skeleton | Orc => Medium,
            Minotaur => Tank,
            BigWorn | BigSpider => Dps,
        }
    }
}
impl EnemyType {
    pub fn variants(self) -> &'static [EnemyKind] {
        match self {
            Small  => &[Bat, Spider],
            Medium => &[Skeleton, Orc],
            Tank   => &[Minotaur],
            Dps    => &[BigWorn, BigSpider],
        }
    }
    pub fn skills(&self, damage_type: Element) -> Vec<Skill> {
        match self {
            Small  => vec![Skill::from_enemy(SkillKind::Attack, 25, |hooks| add_damage(hooks, 20., damage_type))],
            Medium => vec![Skill::from_enemy(SkillKind::Attack, 25, |hooks| add_damage(hooks, 40., damage_type))],
            Tank   => vec![Skill::from_enemy(SkillKind::Attack, 50, |hooks| add_damage(hooks, 60., damage_type))],
            Dps    => vec![Skill::from_enemy(SkillKind::Attack, 20, |hooks| { add_damage(hooks, 60., damage_type); add_pen(hooks, 10.); })],
        }
    }
    pub fn stats(&self) -> CharStats {
        match self {
            Small => CharStats {
                max_health: 100.,
                resistances: Default::default(),
                heal_power: 1.0,
                shield_power: 1.0,
                cdr: 0,
                tick_rate: 1,
            },
            Medium => CharStats {
                max_health: 500.,
                resistances: Default::default(),
                heal_power: 1.0,
                shield_power: 1.0,
                cdr: 0,
                tick_rate: 1,
            },
            Tank => CharStats {
                max_health: 1500.,
                resistances: Elemental::from(10.),
                heal_power: 1.5,
                shield_power: 1.5,
                cdr: 0,
                tick_rate: 1,
            },
            Dps => CharStats {
                max_health: 250.,
                resistances: Default::default(),
                heal_power: 1.0,
                shield_power: 1.0,
                cdr: 0,
                tick_rate: 1,
            },
        }
    }
}

impl EnemyKind {
    pub fn etype(self) -> EnemyType {
        self.into()
    }
    pub fn image(&self) -> Image<'_> {
        match self {
            Bat       => Spritesheet::Bat.get_sprite_2(0, 0),
            BigWorn   => Spritesheet::BigWorn.get_sprite_2(0, 0),
            Minotaur  => Spritesheet::Minotaur.get_sprite_2(0, 0),
            Orc       => Spritesheet::Orc.get_sprite_2(0, 5),
            Skeleton  => Spritesheet::Skeleton.get_sprite_2(0, 0),
            Spider    => Spritesheet::Spider.get_sprite_2(0, 0),
            BigSpider => Spritesheet::BigSpider.get_sprite_2(0, 1),
        }
    }
    pub fn idle_animation(&self) -> Animation {
        match self {
            Bat       => Animation::BatIdle,
            BigWorn   => Animation::BigWormIdle,
            Minotaur  => Animation::MinotaurIdle,
            Orc       => Animation::OrcIdle,
            Skeleton  => Animation::SkeletonIdle,
            Spider    => Animation::SpiderIdle,
            BigSpider => Animation::BigSpiderIdle,
        }
    }
    pub fn attack_animation(&self) -> Animation {
        match self {
            Bat       => Animation::BatAttack,
            BigWorn   => Animation::BigWornAttack,
            Minotaur  => Animation::MinotaurAttack,
            Orc       => Animation::OrcAttack,
            Skeleton  => Animation::SkeletonAttack,
            Spider    => Animation::SpiderAttack,
            BigSpider => Animation::BigSpiderAttack,
        }
    }
}

fn add_damage(hooks: &mut CombatHooks, damge: f32, damage_type: Element) {
    hooks.on_pre_hit(move |attack: &mut PreHit, _skill: &Skill, _user: &Combatant, _target: &Combatant| {
        attack.damage.set(attack.damage.get(damage_type) + damge, damage_type);
    });
}
fn add_pen(hooks: &mut CombatHooks, pen: f32) {
    hooks.on_pre_hit(move |attack: &mut PreHit, _skill: &Skill, _user: &Combatant, _target: &Combatant| {
        attack.penetration = attack.penetration + pen;
    });
}