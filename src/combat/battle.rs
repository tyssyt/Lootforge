use crate::{combat::enemy::EnemyKind, dungeon::{dungeon_data::DungeonTick}, equipment::wardrobe::EquipmentSet, prelude::*};
use super::{combatant::Combatant};

#[derive(Debug)]
pub struct Battle {
    pub tick: u64,
    pub fighter: Combatant,
    // ranger
    // mage
    pub enemies: Vec<Combatant>,
}

#[apply(UnitEnum)]
pub enum BattleResult {
    Ongoing,
    Won,
    Lost,
}

impl Battle {
    pub fn new(equip: &EquipmentSet, enemies: Vec<EnemyKind>, rng: &mut impl Rng) -> Self {
        Self {
            tick: 0,
            fighter: Combatant::fighter(&equip.fighter_equip),
            enemies: Self::enemies(enemies, 1, rng),
        }
    }

    pub fn next(&mut self, enemies: Vec<EnemyKind>, depth: u16, rng: &mut impl Rng) {
        self.tick = 0;
        self.fighter.transfer();
        self.enemies = Self::enemies(enemies, depth, rng);
    }

    fn enemies(enemies: Vec<EnemyKind>, depth: u16, rng: &mut impl Rng) -> Vec<Combatant> {
        enemies.into_iter()
            .enumerate()
            .map(|(i, kind)| Combatant::enemy(kind, i as u8, depth, rng))
            .collect()
    }

    pub fn start(&mut self) {
        self.fighter.combat_start();
        self.enemies.iter_mut().for_each(|e| e.combat_start());
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
