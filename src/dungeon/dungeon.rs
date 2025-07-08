use rand_chacha::ChaCha12Rng;

use crate::dungeon::dungeon_data::DungeonTick;
use crate::dungeon::floor::{Floor, LevelTick};
use crate::prelude::*;
use crate::{
    equipment::wardrobe::{EquipmentSet, OwningEquipmentSet},
    panels::dungeon::dungeon::Background,
};

use super::reward::RewardChest;

#[derive(Debug)]
pub struct Dungeon {
    pub tick: u64,
    pub area: Area,
    pub floor: Floor,
    pub finished: bool,
    pub cancelled: bool,
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
    pub fn dummy() -> Self {
        // bit of a hack to start the game with a "finished" run
        Self {
            tick: 0,
            area: Area {
                background: Default::default(),
            },
            floor: Floor::dummy(),
            finished: true,
            cancelled: false,
            starting_equip: OwningEquipmentSet::default(),
            rng: ChaCha12Rng::from_os_rng(),
        }
    }
    pub fn new(equip: &EquipmentSet, seed: [u8; 32]) -> Self {
        let mut rng = ChaCha12Rng::from_seed(seed);
        Self {
            tick: 0,
            area: Area::new(&mut rng),
            floor: Floor::new(equip, &mut rng),
            finished: false,
            cancelled: false,
            starting_equip: OwningEquipmentSet::from(equip),
            rng,
        }
    }
}

impl Area {
    pub fn new(rng: &mut impl Rng) -> Area {
        Self {
            background: *Background::VARIANTS.choose(rng).unwrap(),
        }
    }
}

impl Dungeon {
    pub fn tick(&mut self) -> (Option<DungeonTick>, Option<RewardChest>) {
        if self.finished {
            return (None, None);
        }
        if self.cancelled {
            self.finished = true;
            return (
                None,
                Some(RewardChest::from(&mut self.rng, self.floor.depth - 1)),
            );
        }

        self.tick += 1;
        
        if self.tick == 1 {
            return (Some(DungeonTick { new_battle: true, skills: Vec::new() }), None);
        }

        match self.floor.tick(&mut self.rng) {
            LevelTick::Waiting => (None, None),
            LevelTick::DungeonTick(dungeon_tick) => (Some(dungeon_tick), None),
            LevelTick::Lost(depth) => {
                self.finished = true;
                (None, Some(RewardChest::from(&mut self.rng, depth - 1)))
            },
        }
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }
}
