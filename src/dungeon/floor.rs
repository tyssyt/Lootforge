use crate::combat::enemy::{EnemyKind, EnemyType};
use crate::dungeon::encounter::Encounter;
use crate::equipment::wardrobe::EquipmentSet;
use crate::prelude::*;

use crate::{
    combat::battle::{Battle, BattleResult},
    dungeon::dungeon_data::DungeonTick,
};

#[derive(Debug)]
pub struct Floor {
    pub depth: u16,
    battle_counter: u16,

    enemy_variation: EnemyVariation,
    encounters: Vec<Encounter>,

    pub battle: Battle,

    pub transition: Option<u32>,
}
impl Floor {
    pub fn dummy() -> Self {
        // bit of a hack to start the game with a "finished" run
        let mut battle = Battle::new(&Default::default(), Vec::new(), &mut rand::rng());
        battle.fighter.health = 0.;

        Self {
            depth: 0,
            battle_counter: 0,
            enemy_variation: EnemyVariation::All,
            encounters: Vec::new(),
            battle,
            transition: None,
        }
    }
    pub fn new(equip: &EquipmentSet, rng: &mut impl Rng) -> Self {
        let enemy_variation = EnemyVariation::new(rng);
        let encounters = Encounter::generate_floor(1, rng);
        let battle = Battle::new(equip, enemy_variation.get_all(&encounters[0].enemies, rng), rng);
        Self {
            depth: 1,
            battle_counter: 1,
            enemy_variation,
            encounters,
            battle,
            transition: Some(Self::TRANSITION_TIME),
        }
    }
}

impl Floor {
    pub const TRANSITION_TIME: u32 = 50;

    pub fn tick(&mut self, rng: &mut impl Rng) -> LevelTick {
        if let Some(ref mut transition) = self.transition {
            *transition -= 1;
            if *transition == 0 {
                self.transition = None;
                self.battle.start();
            }
            return LevelTick::Waiting;
        }

        match self.battle.result() {
            BattleResult::Ongoing => LevelTick::DungeonTick(self.battle.tick()),
            BattleResult::Won => {
                self.advance(rng);
                LevelTick::DungeonTick(DungeonTick { new_battle: true, skills: Vec::new() })
            }
            BattleResult::Lost => LevelTick::Lost(self.depth),
        }
    }

    pub fn advance(&mut self, rng: &mut impl Rng) {
        self.transition = Some(Self::TRANSITION_TIME);
        if self.battle_counter as usize >= self.encounters.len() {
            self.battle_counter = 1;
            self.depth += 1;
            self.enemy_variation = EnemyVariation::new(rng);
            self.encounters = Encounter::generate_floor(self.depth, rng);
        } else {
            self.battle_counter += 1;
        }

        let enemies = self.enemy_variation.get_all(&self.encounters[self.battle_counter as usize -1].enemies, rng);
        self.battle.next(enemies, self.depth, rng);
    }
}

#[apply(Enum)]
pub enum LevelTick {
    Waiting,
    DungeonTick(DungeonTick),
    Lost(u16),
}

#[apply(Enum)]
enum EnemyVariation {
    Fixed(EnemyKind, EnemyKind, EnemyKind, EnemyKind),
    All,
}
impl EnemyVariation {
    fn new(rng: &mut impl Rng) -> Self {
        if rng.random_bool(1./5.) {
            Self::All
        } else {
            Self::Fixed(
                *EnemyType::Small.variants().choose(rng).unwrap(),
                *EnemyType::Medium.variants().choose(rng).unwrap(),
                *EnemyType::Tank.variants().choose(rng).unwrap(),
                *EnemyType::Dps.variants().choose(rng).unwrap(),
            )
        }
    }
    fn get(&self, etype: EnemyType, rng: &mut impl Rng) -> EnemyKind {
        match self {
            EnemyVariation::Fixed(small, medium, tank, dps) => {
                match etype {
                    EnemyType::Small => *small,
                    EnemyType::Medium => *medium,
                    EnemyType::Tank => *tank,
                    EnemyType::Dps => *dps,
                }
            },
            EnemyVariation::All => *etype.variants().choose(rng).unwrap(),
        }
    }
    fn get_all(&self, types: &Vec<EnemyType>, rng: &mut impl Rng) -> Vec<EnemyKind> {
        types.iter().map(|&t| self.get(t, rng)).collect()
    }
}