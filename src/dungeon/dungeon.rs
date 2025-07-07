use rand_chacha::ChaCha12Rng;

use crate::{
    combat::{
        battle::{Battle, BattleResult},
        skill::skill::SkillStats,
    },
    equipment::wardrobe::{EquipmentSet, OwningEquipmentSet},
    panels::dungeon::Background,
    prelude::*,
};

use super::reward::RewardChest;

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
}

#[derive(Debug)]
pub struct Dungeon {
    pub tick: u64,
    pub area: Area,
    pub battle: Battle,
    pub depth: u16,
    pub transition: Option<u32>,
    pub finished: bool,
    pub starting_equip: OwningEquipmentSet,
    pub rng: ChaCha12Rng,
    // run stats
}

#[derive(Debug)]
pub struct Area {
    pub background: Background,
    // background
    // like theming / element ()
    // mod pool (I mean once we have the enemy pool we don't need that)
    // enemy pool
}

impl Dungeon {
    pub const TRANSITION_TIME: u32 = 50;

    pub fn dummy() -> Self {
        // bit of a hack to start the game with a "finished" run
        let mut battle = Battle::new(&mut rand::rng(), &Default::default());
        battle.fighter.health = 0.;

        Self {
            tick: 0,
            area: Area { background: Default::default() },
            battle,
            depth: 0,
            transition: None,
            finished: true,
            starting_equip: OwningEquipmentSet::default(),
            rng: ChaCha12Rng::from_os_rng(),
        }
    }
    pub fn new(equip: &EquipmentSet, seed: [u8; 32]) -> Self {
        let mut rng = ChaCha12Rng::from_seed(seed);
        Self {
            tick: 0,
            area: Area::new(&mut rng),
            battle: Battle::new(&mut rng, equip),
            depth: 1,
            transition: Some(Self::TRANSITION_TIME),
            finished: false,
            starting_equip: OwningEquipmentSet::from(equip),
            rng,
        }
    }
}

impl Area {
    pub fn new(rng: &mut impl Rng) -> Area {
        Self { background: *Background::VARIANTS.choose(rng).unwrap() }
    }
}

impl DungeonData {
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

impl Dungeon {
    pub fn tick(&mut self) -> (Option<DungeonTick>, Option<RewardChest>) {
        if self.finished {
            return (None, None);
        }

        self.tick += 1;

        if let Some(ref mut transition) = self.transition {
            *transition -= 1;
            if *transition == 0 {
                self.transition = None;
                self.battle.start();
            }
            return (None, None);
        }

        let result = self.battle.result();
        match result {
            BattleResult::Ongoing => (Some(self.battle.tick()), None),
            BattleResult::Won => {
                self.depth += 1;
                self.battle.next(&mut self.rng, self.depth);
                self.transition = Some(Self::TRANSITION_TIME);
                (None, None)
            }
            BattleResult::Lost => {
                self.finished = true;
                (None, Some(RewardChest::from(&mut self.rng, self.depth - 1)))
            }
        }
    }
}

#[apply(Default)]
pub struct DungeonTick {
    pub skills: Vec<SkillStats>,
}
