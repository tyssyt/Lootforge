use crate::prelude::*;

use super::{dungeon::Dungeon, reward::RewardChest};
use crate::{combat::skill::skill::SkillStats, equipment::wardrobe::EquipmentSet};

#[derive(Debug, SmartDefault)]
pub struct DungeonData {
    #[default(Dungeon::dummy())]
    pub cur: Dungeon,
    pub rewards: Vec<RewardChest>,
    pub auto_restart: bool,
}
impl DungeonData {
    pub fn restart(&mut self, equipment: &EquipmentSet) {
        let mut seed = [0; 32];
        rand::rng().fill_bytes(&mut seed);
        self.cur = Dungeon::new(equipment, seed);
    }

    pub fn tick(&mut self, equipment: &EquipmentSet) -> Option<DungeonTick> {
        let (tick, reward) = self.cur.tick();
        if let Some(reward) = reward {
            if reward.items.len() > 0 {
                self.rewards.push(reward);
            }
            if self.auto_restart {
                self.restart(equipment);
            }
        }
        tick
    }
}

#[apply(Default)]
pub struct DungeonTick {
    pub new_battle: bool,
    pub skills: Vec<SkillStats>,
}
