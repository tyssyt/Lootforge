use std::rc::Rc;

use crate::combat::combatant::{CharStats, Combatant};
use crate::combat::skill::{attack, defend};
use crate::equipment::equipment::{Equip, EquipEnum};
use crate::item::Item;
use crate::{combat::hooks::CombatHooks, item::ItemType};
use crate::prelude::*;

use super::{attack::AttackStats, defend::DefStats, targeting::Targeting};

use SkillKind::*;

#[derive(Debug)]
pub struct Skill {
    pub source: SkillSource,
    pub kind: SkillKind, // TODO integrate this to source, couse for item source I can derive it
    pub targeting: Targeting,
    pub hooks: CombatHooks,
    pub cd: u16,
    pub uses: u16,
}
impl Skill { // constructors
    pub fn transfer(&mut self) {
        self.cd = self.cooldown();
        self.uses = 0;
    }
    pub fn from_item(item: Rc<Item>, equip: &EquipEnum) -> Option<Self> {
        if let Some(kind) = SkillKind::from_item_type(item.item_type) {
            let mut hooks = CombatHooks::default();
            item.mods.iter().for_each(|m| m.register(&mut hooks, &item, equip));

            let targeting = item.targeting
                .or_else(|| equip.get_linked_item(&item).upgrade().and_then(|i| i.targeting))
                .unwrap_or(kind.default_targeting());

            let mut res = Self {
                source: SkillSource::Item(item.id, item.item_type),
                kind,
                targeting,
                hooks: hooks,
                cd: 0,
                uses: 0,
            };
            res.cd = res.cooldown();
            Some(res)
        } else {
            None
        }
    }
    pub fn from_enemy(kind: SkillKind, cd: u16) -> Self {
        Self {
            source: SkillSource::Enemy(cd),
            kind,
            targeting: Targeting::First, // TODO aggro
            hooks: CombatHooks::default(), // TODO add enemy mods here
            cd,
            uses: 0,
        }
    }
}

impl Skill {
    pub fn cooldown(&self) -> u16 {
        let base = match &self.source {
            SkillSource::Item(..) => self.kind.base_cooldown(),
            SkillSource::Enemy(cd) => *cd,
        };

        let mut char = CharStats::default();
        // cdr can currently only roll on the skill giving items, so we only nee to check our mods
        self.hooks.char(&mut char);

        let reduced_ticks = base * char.cdr / 100;
        base - reduced_ticks
    }

    pub fn image(&self) -> Image<'_> {
        match &self.source {
            SkillSource::Item(_, item_type) => item_type.image(),
            SkillSource::Enemy(_) => panic!("enemy skills do not have images"),
        }
    }

    pub fn ready(&self) -> bool {
        self.cd == 0
    }

    pub fn tick(&mut self) {
        if self.cd > 0 {
            self.cd -= 1;
        }
    }

    pub fn trigger<'a, 'b>(
        &mut self,
        user: &mut Combatant,
        allies: &'a mut Vec<&'b mut Combatant>,
        enemies: &'a mut Vec<&'b mut Combatant>,
        reset_cooldown: bool,
    ) -> SkillStats {        
        use SkillKind::*;
        let skill_stats = match self.kind {
            Attack =>    SkillStats::Attack(self.source.clone(), attack::attack_single(self, user, allies, enemies)),
            AoeAttack => SkillStats::Attack(self.source.clone(), attack::attack_aoe(self, user, allies, enemies)),
            Defend =>    todo!(), // TODO
        };
        if reset_cooldown {
            self.cd = self.cooldown();
        }
        skill_stats
    }
    pub fn trigger_against_target(
        &mut self,
        user: &mut Combatant,
        target: &mut Combatant,
        reset_cooldown: bool,
    ) -> SkillStats {
        use SkillKind::*;
        let skill_stats = match self.kind {
            Attack | AoeAttack => SkillStats::Attack(self.source.clone(), attack::attack_target(self, user, target)),
            Defend => SkillStats::Defend(self.source.clone(), defend::defend(self, user)),
        };
        if reset_cooldown {
            self.cd = self.cooldown();
        }
        self.uses += 1;
        skill_stats
    }
}

#[apply(UnitEnum)]
pub enum SkillKind {
    Attack ,
    AoeAttack,
    Defend,
    // the 3 support skills
}
impl SkillKind {
    pub fn from_item_type(item_type: ItemType) -> Option<Self> {
        use ItemType::*;
        match item_type {
            Axe | Crossbow => Some(Attack), // mage gem
            Sword | Bow => Some(AoeAttack), // mage gem
            // 3 support skills
            Helmet => Some(Defend),
            _ => return None,
        }
    }

    pub fn default_targeting(&self) -> Targeting {
        match self {
            SkillKind::Attack | AoeAttack => Targeting::First,
            Defend => Targeting::OnAttack,
        }
    }

    pub fn base_cooldown(&self) -> u16 {
        match self {
            SkillKind::Attack => 20,
            AoeAttack         => 30,
            SkillKind::Defend => 70,
        }
    }
}

#[apply(Enum)]
#[derive(PartialEq)]
pub enum SkillSource {
    Item(usize, ItemType),
    Enemy(u16),
}

#[apply(Enum)]
pub enum SkillStats {
    Attack(SkillSource, AttackStats),
    Defend(SkillSource, DefStats),
}
impl SkillStats {
    pub fn source(&self) -> &SkillSource {
        match self {
            SkillStats::Attack(skill_source, _) => skill_source,
            SkillStats::Defend(skill_source, _) => skill_source,
        }
    }
}