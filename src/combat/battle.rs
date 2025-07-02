use crate::{dungeon::dungeon::DungeonTick, equipment::wardrobe::EquipmentSet, prelude::*};
use super::{combatant::Combatant, enemy::EnemyKind};

pub struct Battle {
    pub tick: u64,
    pub fighter: Combatant,
    // ranger
    // mage
    pub enemies: Vec<Combatant>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BattleResult {
    Ongoing,
    Won,
    Lost,
}

impl Battle {
    pub fn new(rng: &mut impl Rng, equip: &EquipmentSet) -> Self {
        Self {
            tick: 0,
            fighter: Combatant::fighter(&equip.fighter_equip),
            enemies: vec![
                Combatant::enemy(rng, EnemyKind::BigWorn, 0, 1),
            ],
        }
    }

    pub fn next(&mut self, rng: &mut impl Rng, depth: u16) {
        self.tick = 0;
        self.fighter.transfer();
        self.enemies.clear();
        for i in 0..((depth + 4) / 5).at_most(4) {
            self.enemies.push(Combatant::enemy(rng, EnemyKind::BigWorn, i as u8, depth));
        }
    }

    pub fn tick(&mut self) -> DungeonTick {
        self.tick += 1;
        let mut tick_info = DungeonTick::default();

        if self.fighter.alive() {
            let skill_stats = self.fighter.tick(
                self.tick,
                vec![],
                self.enemies.iter_mut().filter(|e| e.alive()).collect(),
            );
            skill_stats.map(|skill| tick_info.skills.push(skill));
        }

        for enemy in &mut self.enemies {
            if enemy.alive() {
                let skill_stats = enemy.tick(
                    self.tick,
                    vec![],
                    vec![&mut self.fighter]
                );
                skill_stats.map(|skill| tick_info.skills.push(skill));
            }
        }

        tick_info
    }

    pub fn result(&self) -> BattleResult {
        if !self.fighter.alive() {
            BattleResult::Lost
        } else if self.enemies.iter().all(|e| !e.alive()) {
            BattleResult::Won
        } else {
            BattleResult::Ongoing
        }
    }
}
