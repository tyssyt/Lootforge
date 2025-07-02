
/*
    Option 1: Serde + this file as spec + Postcard
    Option 2: do not have this file, have something that writes u8s and then give that to backend

    either way I do compress it before storing

*/

struct Save {
    timestamp: u128,
    stash: Vec<Item>,
    equip: [Equip;9],
    chests: Vec<Chest>,
    dungeon: Dungeon,
}

struct Item {
    item_type: u8,
    targeting: u8,
    mods: Vec<ItemMod>
}

struct ItemMod {
    mod_type: u16,
    roll: u16,
}

struct Equip {
    fighter: [u32;9],
    ranger: [u32;9],
    mage: [u32;9],
}

struct Chest {
    items: Vec<Item>,
}

struct Dungeon {
    auto_restart: bool,
    finished: bool,
    depth: u16,
    transition: u32,
    area: u8,
    // rng state maybe serde? otherwise get_seed, get_word_pos, get_stream
    battle: Battle,
}

struct Area {

}

struct Battle {
    fighter: Combatant<ExplorerSkill>,
    ranger: Combatant<ExplorerSkill>,
    mage: Combatant<ExplorerSkill>,
    enemies: Vec<Combatant<EnemySkill>>,
}

struct Combatant <S> {
    kind: u16,
    health: u16,
    shield: u16,
    buffs: Vec<u8>, // placeholder
    skills: Vec<S>,
    passive_mods: Vec<ItemMod>,
}

struct ExplorerSkill {
    item_type: u8,
    mods: Vec<ItemMod>,
    cd: u16,
}

struct EnemySkill {
    enemy_kind: u16,
    skill_kind: u8,
    base_cd: u16,
    cd: u16,
}

/*
Format
4 bytes magic number
u8 version


// Current Dungeon
*area info* <-- currently just the background, but will be more later


*/