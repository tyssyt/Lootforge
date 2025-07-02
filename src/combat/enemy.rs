use crate::prelude::*;
use EnemyKind::*;

use crate::{combat::skill::skill::{Skill, SkillKind}, panels::animation::Animation};

#[derive(Clone, Copy, Debug)]
pub enum EnemyKind {
    BigWorn,
}

impl EnemyKind {
    pub fn skills(&self) -> Vec<Skill> {
        match *self {
            BigWorn => vec![Skill::from_enemy(SkillKind::Attack, 25)],
        }
    }
    // something that gives me attack img (needs to be at least 3 frames, I can have logic that can "charge up" before the timer hits zero and frames after, but those can get cutt off)
    // something that gives me idle img (animation can be pretty much arbitrary long, I guess I can just use tick mod anim length)

    // prjectile for ranged

    pub fn image(&self) -> Image<'_> {
        // TODO why does this not work
        // self.idle_animation().frame(0)
        match *self {
            BigWorn => Animation::BigWormIdle.frame(0),
        }
    }
    pub fn idle_animation(&self) -> Animation {
        match *self {
            BigWorn => Animation::BigWormIdle,
        }
    }
}
