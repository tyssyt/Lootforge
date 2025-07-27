use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
    u8,
};

use crate::{
    prelude::*,
    combat::skill::targeting::Targeting,
    dungeon::{dungeon::Dungeon, dungeon_data::DungeonData, floor::Floor, reward::RewardChest},
    elemental::Element,
    equipment::{
        equipment::{CommonEquip, FighterEquip},
        wardrobe::{EquipmentSet, Wardrobe},
    },
    item::{Item, ItemRef, ItemType, ItemUsers},
    mods::RolledMod,
    stash::stash::Stash,
    timekeeper::Timekeeper,
    LootforgeApp,
};
use web_time::SystemTime;

use super::{ser, storage_manager::StorageManager};

const CRC: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

pub fn validate(bytes: &[u8]) -> Option<u64> {
    if bytes.len() < 1000 {
        return None;
    }
    let check_sum = CRC.checksum(&bytes[..bytes.len() - 4]).to_le_bytes();
    if &bytes[bytes.len() - 4..] != check_sum {
        return None;
    }

    Some(u64::from_le_bytes(bytes[9..17].try_into().ok()?))
}

// TODO if I use a trait, I don't have to make everything pub :)

pub fn ser(app: &LootforgeApp, epoch_millis: u64) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(1000);

    bytes.extend_from_slice(&ser::MAGIC);
    ser_u8(&mut bytes, 1);
    ser_u64(&mut bytes, epoch_millis);
    let items = ser_stash(&mut bytes, &app.stash);
    ser_wardrobe(&mut bytes, &app.wardrobe, &items);
    ser_dungeon_data(&mut bytes, &app.dungeon);

    let check_sum = CRC.checksum(&bytes).to_le_bytes();
    bytes.extend_from_slice(&check_sum);
    bytes
}

pub fn deser(storage_manager: StorageManager, mut bytes: &[u8]) -> Option<LootforgeApp> {
    let ts = validate(bytes)?;
    let mut timekeeper = Timekeeper::default();
    timekeeper.last_save_sim = SystemTime::UNIX_EPOCH + Duration::from_millis(ts);

    bytes = &bytes[17..bytes.len() - 4];

    let stash = deser_stash(&mut bytes)?;
    let wardrobe = deser_wardrobe(&mut bytes, &stash)?;
    let dungeon_data = deser_dungeon_data(&mut bytes)?;

    Some(LootforgeApp {
        timekeeper,
        storage_manager,
        stash,
        wardrobe,
        dungeon: dungeon_data,
        ..Default::default()
    })
}

fn ser_stash(bytes: &mut Vec<u8>, stash: &Stash) -> BTreeMap<usize, u32> {
    let mut items = BTreeMap::new();

    ser_u32(bytes, stash.items().len() as u32);
    for (i, item) in stash.items().iter().enumerate() {
        ser_item(bytes, item.as_ref());
        if item.users.any_wardrobe.get() {
            items.insert(item.id, (i + 1) as u32);
        }
    }
    items
}
fn deser_stash(bytes: &mut &[u8]) -> Option<Stash> {
    let mut stash = Stash::default();
    let len = deser_u32(bytes)?;
    for _ in 0..len {
        stash.add(deser_item(bytes)?);
    }
    Some(stash)
}

fn ser_wardrobe(bytes: &mut Vec<u8>, wardrobe: &Wardrobe, items: &BTreeMap<usize, u32>) {
    let ser_slot = |bytes: &mut Vec<u8>, slot: &Weak<Item>| {
        if let Some(item) = slot.upgrade() {
            ser_u32(bytes, items[&item.id]);
        } else {
            ser_u32(bytes, 0);
        }
    };

    ser_u8(bytes, wardrobe.equipped as u8);
    for set in &wardrobe.sets {
        ser_equipment_set(bytes, set, ser_slot);
    }
}
fn deser_wardrobe(bytes: &mut &[u8], stash: &Stash) -> Option<Wardrobe> {
    let deser_slot = |bytes: &mut &[u8]| {
        let id = deser_u32(bytes)? as usize;
        if id == 0 {
            Some(Weak::new())
        } else {
            Some(Rc::downgrade(&stash.find(id)?))
        }
    };

    let equipped = deser_u8(bytes)? as usize;
    // TODO maybe use https://doc.rust-lang.org/stable/std/array/fn.try_from_fn.html if it becomes stable
    let sets = [
        deser_equipment_set(bytes, deser_slot)?,
        deser_equipment_set(bytes, deser_slot)?,
        deser_equipment_set(bytes, deser_slot)?,
        deser_equipment_set(bytes, deser_slot)?,
        deser_equipment_set(bytes, deser_slot)?,
        deser_equipment_set(bytes, deser_slot)?,
        deser_equipment_set(bytes, deser_slot)?,
        deser_equipment_set(bytes, deser_slot)?,
        deser_equipment_set(bytes, deser_slot)?,
    ];

    sets.iter().enumerate().for_each(|(i, set)| {
        set.iter().for_each(|item| {
            item.upgrade().map(|item| item.users.add_wardrobe(i));
        });
    });

    let mut wardrobe = Wardrobe { sets, equipped: (equipped + 1) % 9 };
    wardrobe.set_equipped(equipped); // TODO this sets the equipped flag, prolly can do that a bit nicer
    Some(wardrobe)
}

fn ser_equipment_set(bytes: &mut Vec<u8>, set: &EquipmentSet, ser_slot: impl Fn(&mut Vec<u8>, &ItemRef)) {
    ser_slot(bytes, &set.fighter_equip.weapons[0]);
    ser_slot(bytes, &set.fighter_equip.weapons[1]);
    ser_slot(bytes, &set.fighter_equip.shield);
    ser_slot(bytes, &set.fighter_equip.common.helmet);
    ser_slot(bytes, &set.fighter_equip.common.armor);
    ser_slot(bytes, &set.fighter_equip.common.gloves);
    ser_slot(bytes, &set.fighter_equip.common.rings[0]);
    ser_slot(bytes, &set.fighter_equip.common.rings[1]);
    ser_slot(bytes, &set.fighter_equip.common.rings[2]);

    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);

    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
    ser_u32(bytes, 0);
}
fn deser_equipment_set(bytes: &mut &[u8], mut deser_slot: impl FnMut(&mut &[u8]) -> Option<ItemRef>) -> Option<EquipmentSet> {
    let set = EquipmentSet {
        fighter_equip: FighterEquip {
            weapons: [deser_slot(bytes)?, deser_slot(bytes)?],
            shield: deser_slot(bytes)?,
            common: CommonEquip {
                helmet: deser_slot(bytes)?,
                armor: deser_slot(bytes)?,
                gloves: deser_slot(bytes)?,
                rings: [deser_slot(bytes)?, deser_slot(bytes)?, deser_slot(bytes)?],
            },
        },
    };
    for _ in 0..18 {
        deser_u32(bytes)?;
    }
    Some(set)
}

fn ser_rewards(bytes: &mut Vec<u8>, rewards: &BTreeMap<u16, Vec<RewardChest>>) {
    let rewards_len: usize = rewards.values().map(|r| r.len()).sum();
    ser_u32(bytes, rewards_len as u32);
    for chest in rewards.values().flatten() {
        ser_u16(bytes, chest.depth);
        ser_u16(bytes, chest.items.len() as u16);
        for item in &chest.items {
            ser_item(bytes, item);
        }
    }
}
fn deser_rewards(bytes: &mut &[u8]) -> Option<BTreeMap<u16, Vec<RewardChest>>> {    
    let mut rewards: BTreeMap<u16, Vec<RewardChest>> = BTreeMap::new();
    let len = deser_u32(bytes)?;
    for _ in 0..len {
        let depth = deser_u16(bytes)?;
        let len = deser_u16(bytes)?;
        let items = (0..len).map(|_| deser_item(bytes)).collect::<Option<_>>()?;
        rewards.entry(depth).or_default().push(RewardChest { depth, items });
    }
    Some(rewards)
}

fn ser_dungeon_data(bytes: &mut Vec<u8>, dungeon_data: &DungeonData) {
    ser_dungeon(bytes, &dungeon_data.cur);
    ser_rewards(bytes, &dungeon_data.rewards);
    ser_u8(bytes, dungeon_data.auto_restart as u8);
}
fn deser_dungeon_data(bytes: &mut &[u8]) -> Option<DungeonData> {
    Some(DungeonData {
        cur: deser_dungeon(bytes)?,
        rewards: deser_rewards(bytes)?,
        auto_restart: deser_u8(bytes)? != 0,
    })
}

fn ser_dungeon(bytes: &mut Vec<u8>, dungeon: &Dungeon) {
    // TODO ser the auto restart bit outside
    ser_u8(bytes, dungeon.finished as u8);
    if dungeon.finished {
        return;
    }

    fn ser_dungeon_item(bytes: &mut Vec<u8>, item: &Weak<Item>) {
        if let Some(item) = item.upgrade() {
            ser_item(bytes, &item);
        } else {
            ser_u8(bytes, 0);
        }
    }

    ser_equipment_set(bytes, &dungeon.starting_equip.equipment_set, ser_dungeon_item);
    ser_u64(bytes, dungeon.tick);
    bytes.extend_from_slice(&dungeon.rng.get_seed());
    ser_u32(bytes, dungeon_checksum(&dungeon.floor))
}
fn deser_dungeon(bytes: &mut &[u8]) -> Option<Dungeon> {
    let finished = deser_u8(bytes)? != 0;
    if finished {
        return Some(Dungeon::dummy());
    }

    let mut items: Vec<Rc<Item>> = Vec::new();
    let deser_dungeon_item = |bytes: &mut &[u8]| {
        if *bytes.first()? == 0 {
            deser_u8(bytes)?;
            return Some(Weak::new());
        }

        static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

        let mut item = deser_item(bytes)?;
        item.id = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let item = Rc::new(item);
        let weak = Rc::downgrade(&item);
        items.push(item);
        Some(weak)
    };

    let starting_equip = deser_equipment_set(bytes, deser_dungeon_item)?;
    let tick = deser_u64(bytes)?;
    let seed = deser_bytes(bytes)?;

    let mut dungeon = Dungeon::new(&starting_equip, seed);

    for _ in 0..tick {
        dungeon.tick();
    }

    let checksum = deser_u32(bytes)?;
    if checksum != dungeon_checksum(&dungeon.floor) {
        error!("Deserialized Dungeon has diverging checksum after simulating {} ticks", tick);
        return None;
    }

    Some(dungeon)
}

fn dungeon_checksum(level: &Floor) -> u32 {
    let mut checksum = Vec::new();

    ser_u16(&mut checksum, level.depth);
    ser_u32(&mut checksum, level.transition.unwrap_or(0));
    ser_u64(&mut checksum, level.battle.tick);

    for combatant in [&level.battle.fighter, &level.battle.fighter, &level.battle.fighter].into_iter().chain(level.battle.enemies.iter()) {
        ser_f32(&mut checksum, combatant.health);
        ser_f32(&mut checksum, combatant.shield);
        for skill in &combatant.skills {
            ser_u16(&mut checksum, skill.cd);
        }
    }

    CRC.checksum(&checksum)
}

// --

fn ser_item(bytes: &mut Vec<u8>, item: &Item) {
    ser_u8(bytes, item.item_type as u8);
    ser_targeting(bytes, &item.targeting);
    ser_mods(bytes, &item.mods);
    ser_u8(bytes, item.rerolled_mod_idx);
}
fn deser_item(bytes: &mut &[u8]) -> Option<Item> {
    Some(Item {
        id: 0,
        item_type: ItemType::from_repr(deser_u8(bytes)?)?,
        targeting: deser_targeting(bytes)?,
        mods: deser_mods(bytes)?,
        rerolled_mod_idx: deser_u8(bytes)?,
        users: ItemUsers::default(),
    })
}

fn ser_targeting(bytes: &mut Vec<u8>, targeting: &Option<Targeting>) {
    use Targeting::*;
    match targeting {
        Some(First)                                    => { ser_u8(bytes, 1); ser_u8(bytes, 0); },
        Some(LowestHealth)                             => { ser_u8(bytes, 2); ser_u8(bytes, 0); },
        Some(LowestEffectiveHealth(element)) => { ser_u8(bytes, 3); ser_u8(bytes, *element as u8); },
        Some(LowestResistance(element))      => { ser_u8(bytes, 4); ser_u8(bytes, *element as u8); },
        Some(HighestRank)                              => { ser_u8(bytes, 5); ser_u8(bytes, 0); },
        Some(LowestRank)                               => { ser_u8(bytes, 6); ser_u8(bytes, 0); },
        Some(RoundRobin(u))                       => { ser_u8(bytes, 7); ser_u8(bytes, *u); },

        Some(Instant)                                  => { ser_u8(bytes, 100); ser_u8(bytes, 0); },
        Some(OnAttack)                                 => { ser_u8(bytes, 101); ser_u8(bytes, 0); },
        None                                           => { ser_u8(bytes, 0); ser_u8(bytes, 0); },
    };
}
fn deser_targeting(bytes: &mut &[u8]) -> Option<Option<Targeting>> {
    use Targeting::*;
    match (deser_u8(bytes)?, deser_u8(bytes)?) {
        (1, _)     => Some(Some(First)),
        (2, _)     => Some(Some(LowestHealth)),
        (3, u) => Some(Some(LowestEffectiveHealth(Element::from_repr(u)?))),
        (4, u) => Some(Some(LowestResistance(Element::from_repr(u)?))),
        (5, _)     => Some(Some(HighestRank)),
        (6, _)     => Some(Some(LowestRank)),
        (7, u) => Some(Some(RoundRobin(u))),
        (0, _)     => Some(None),

        (100, _)   => Some(Some(Instant)),
        (101, _)   => Some(Some(OnAttack)),
        _ => None
    }
}

fn ser_mods(bytes: &mut Vec<u8>, mods: &Vec<RolledMod>) {
    ser_u8(bytes, mods.len() as u8);
    for item_mod in mods {
        ser_u16(bytes, item_mod.mod_id);
        ser_u16(bytes, item_mod.roll);
    }
}
fn deser_mods(bytes: &mut &[u8]) -> Option<Vec<RolledMod>> {
    let len = deser_u8(bytes)?;
    (0..len).map(|_| 
        Some(RolledMod {
            mod_id: deser_u16(bytes)?,
            roll: deser_u16(bytes)?,
        })
    ).collect()
}

// numbers

fn ser_u8(bytes: &mut Vec<u8>, u: u8) {
    bytes.push(u);
}
fn ser_u16(bytes: &mut Vec<u8>, u: u16) {
    bytes.extend_from_slice(&u.to_le_bytes());
}
fn ser_u32(bytes: &mut Vec<u8>, u: u32) {
    bytes.extend_from_slice(&u.to_le_bytes());
}
fn ser_u64(bytes: &mut Vec<u8>, u: u64) {
    bytes.extend_from_slice(&u.to_le_bytes());
}
fn ser_f32(bytes: &mut Vec<u8>, f: f32) {
    bytes.extend_from_slice(&f.to_le_bytes());
}

fn deser_u8(bytes: &mut &[u8]) -> Option<u8> {
    let (first, rest) = bytes.split_first()?;
    *bytes = rest;
    Some(*first)
}
fn deser_bytes<const N: usize>(bytes: &mut &[u8]) -> Option<[u8; N]> {
    let (first, rest) = bytes.split_first_chunk::<N>()?;
    *bytes = rest;
    Some(*first)
}
fn deser_u16(bytes: &mut &[u8]) -> Option<u16> {
    Some(u16::from_le_bytes(deser_bytes(bytes)?))
}
fn deser_u32(bytes: &mut &[u8]) -> Option<u32> {
    Some(u32::from_le_bytes(deser_bytes(bytes)?))
}
fn deser_u64(bytes: &mut &[u8]) -> Option<u64> {
    Some(u64::from_le_bytes(deser_bytes(bytes)?))
}
fn _deser_f32(bytes: &mut &[u8]) -> Option<f32> {
    Some(f32::from_le_bytes(deser_bytes(bytes)?))
}
