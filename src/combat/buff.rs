use std::{fmt::Debug, mem};

use crate::{combat::skill::{hit::{self, Hit}, skill::Skill}, elemental::{Element, Elemental}, prelude::*};

use super::{combatant::{CharStats, Combatant}, skill::{attack::{AttackPostHit, AttackPreHit, ResponsePostHit, ResponsePreHit}, defend::Defend}};

#[derive(Debug)]
pub enum Buff {
    Attuned(AttunedBuff),
    Reverb(ReverbBuff),

    Bleed(BleedDebuff),
    Vulnerable(VulnerableDebuff),
    Confused(ConfusedDebuff),
    Echo(EchoDebuff),

    Lifelink(LifelinkDebuff),
    Incapacitated(IncapacitatedDebuff),
    Dazed(DazedDebuff),
    Soullink(SoullinkDebuff),
}
pub type Debuff = Buff;

#[derive(Debug, Default)]
pub struct Buffs {
    attuned: Vec<AttunedBuff>,
    reverb: Vec<ReverbBuff>,

    bleed: Vec<BleedDebuff>,
    vulnerable: Vec<VulnerableDebuff>,
    confused: Vec<ConfusedDebuff>,
    echo: Vec<EchoDebuff>,
    
    lifelink: Vec<LifelinkDebuff>,
    incapacitated: Vec<IncapacitatedDebuff>,
    dazed: Vec<DazedDebuff>,
    soullink: Vec<SoullinkDebuff>,
}

impl Buffs {
    pub fn add(&mut self, buff: Buff) {
        match buff {
            Buff::Attuned(attuned_buff) => self.attuned.push(attuned_buff),
            Buff::Reverb(reverb_buff) => self.reverb.push(reverb_buff),

            Buff::Bleed(bleed_debuff) => self.bleed.push(bleed_debuff),
            Buff::Vulnerable(vulnerable_debuff) => self.vulnerable.push(vulnerable_debuff),
            Buff::Confused(confused_debuff) => self.confused.push(confused_debuff),            
            Buff::Echo(echo_debuff) => self.echo.push(echo_debuff),

            Buff::Lifelink(lifelink_debuff) => self.lifelink.push(lifelink_debuff),
            Buff::Incapacitated(incapacitated_debuff) => self.incapacitated.push(incapacitated_debuff),
            Buff::Dazed(dazed_debuff) => self.dazed.push(dazed_debuff),
            Buff::Soullink(soullink_debuff) => self.soullink.push(soullink_debuff),
        }
    }

    pub fn tick(owner: &mut Combatant) {
        {
            let mut bleed = mem::take(&mut owner.buffs.bleed);
            BleedDebuff::tick(&mut bleed, owner);
            owner.buffs.bleed = bleed;
        }
        {
            let mut echo = mem::take(&mut owner.buffs.echo);
            EchoDebuff::tick(&mut echo, owner);
            owner.buffs.echo = echo;
        }
        IncapacitatedDebuff::tick(&mut owner.buffs.incapacitated);
    }
    pub fn attacked(&mut self) {
        self.reverb.clear();
        if !self.confused.is_empty() {
            self.confused.remove(0);
        }
        if !self.dazed.is_empty() {
            self.dazed.remove(0);
        }
    }

    pub fn icons(&self) -> impl Iterator<Item = BuffIcon> {
        [
            (self.attuned.is_empty(), AttunedBuff::ICON),
            (self.reverb.is_empty(), ReverbBuff::ICON),

            (self.bleed.is_empty(), BleedDebuff::ICON),
            (self.vulnerable.is_empty(), VulnerableDebuff::ICON),
            (self.confused.is_empty(), ConfusedDebuff::ICON),
            (self.echo.is_empty(), EchoDebuff::ICON),
            
            (self.lifelink.is_empty(), LifelinkDebuff::ICON),
            (self.incapacitated.is_empty(), IncapacitatedDebuff::ICON),
            (self.dazed.is_empty(), DazedDebuff::ICON),
            (self.soullink.is_empty(), SoullinkDebuff::ICON),
        ].into_iter()
            .filter(|(is_empty, _)| !*is_empty)
            .map(|(_, icon)| icon)
    }
    pub fn tooltip(&self, ui: &mut Ui) {
        AttunedBuff::tooltip(&self.attuned, ui);
        ReverbBuff::tooltip(&self.reverb, ui);

        BleedDebuff::tooltip(&self.bleed, ui);
        VulnerableDebuff::tooltip(&self.vulnerable, ui);
        ConfusedDebuff::tooltip(&self.confused, ui);
        EchoDebuff::tooltip(&self.echo, ui);
        
        LifelinkDebuff::tooltip(&self.lifelink, ui);
        IncapacitatedDebuff::tooltip(&self.incapacitated, ui);
        DazedDebuff::tooltip(&self.dazed, ui);
        SoullinkDebuff::tooltip(&self.soullink, ui);
    }
}
impl Buffs {
    pub fn apply_pre_hit(&self, attack: &mut AttackPreHit, skill: &Skill, user: &Combatant, target: &Combatant) {
        self.reverb.iter().for_each(|b| b.apply_pre_hit(attack, skill, user, target));
        self.dazed.first().map(|b| b.apply_pre_hit(attack, skill, user, target));
    }    
    pub fn apply_post_hit(&self, attack: &mut AttackPostHit, skill: &Skill, user: &Combatant, target: &Combatant, hit: &Hit) {
        self.confused.iter().for_each(|b| b.apply_post_hit(attack, skill, user, target, hit));
    }
    pub fn apply_pre_getting_hit(&mut self, attack: &mut AttackPreHit) {
        self.vulnerable.iter().for_each(|b| b.apply_pre_getting_hit(attack));
        self.vulnerable.clear();
    }
    pub fn apply_post_getting_hit(&mut self, attack: &mut AttackPostHit) {
        self.lifelink.iter().for_each(|b| b.apply_post_getting_hit(attack));
        self.lifelink.clear();
        
        self.soullink.iter().for_each(|b| b.apply_post_getting_hit(attack));
        self.soullink.clear();
    }
    pub fn apply_pre_atk(&self, _resp: &mut ResponsePreHit, _skill: &Skill, _user: &Combatant, _attacker: &Combatant) {
    }
    pub fn apply_post_atk(&self, _resp: &mut ResponsePostHit, _skill: &Skill, _user: &Combatant, _attacker: &Combatant, _hit: &Hit) {
    }
    pub fn apply_to_def(&self, _def: &mut Defend, _skill: &Skill, _user: &Combatant) {
    }
    pub fn apply_to_char(&self, char: &mut CharStats) {
        self.attuned.iter().for_each(|b| b.apply_to_char(char));
        self.incapacitated.first().map(|b| b.apply_to_char(char));
    }
}


#[derive(Debug)]
pub struct AttunedBuff {    
    value: f32,
    element: Element,
}
impl Buff {
    pub fn attuned(value: f32, element: Element) -> Self {
        Self::Attuned( AttunedBuff { value, element } )
    }
}
impl AttunedBuff {    
    pub const ICON: BuffIcon = BuffIcon::ResUp;

    fn apply_to_char(&self, char: &mut CharStats) {
        *char.resistances.get_mut(self.element) += self.value;
    }

    pub fn tooltip(buffs: &Vec<Self>, ui: &mut Ui) {
        if buffs.is_empty() {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.add(Self::ICON.image());
            let res: Elemental<f32> = buffs.iter().map(|b| Elemental::default().with(b.value, b.element)).sum();
            ui.label(format!("Add {}/{}/{}/{} Resistance", res.bleed, res.fracture, res.madness, res.void));
        });
    }
}

#[derive(Debug)]
pub struct ReverbBuff {
    damage: Elemental<f32>,
}
impl Buff {
    pub fn reverb(damage: Elemental<f32>) -> Self {
        Self::Reverb( ReverbBuff { damage } )
    }
}
impl ReverbBuff {
    pub const ICON: BuffIcon = BuffIcon::DmgUp;

    fn apply_pre_hit(&self, attack: &mut AttackPreHit, _skill: &Skill, _user: &Combatant, _target: &Combatant) {
        attack.damage = attack.damage + self.damage;
    }    

    pub fn tooltip(buffs: &Vec<Self>, ui: &mut Ui) {
        if buffs.is_empty() {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.add(Self::ICON.image());
            let damage: Elemental<f32> = buffs.iter().map(|b| b.damage).sum();
            ui.label(format!("Add {}/{}/{}/{} Damage to the next Attack", damage.bleed, damage.fracture, damage.madness, damage.void));
        });
    }
}

#[derive(Debug)]
pub struct BleedDebuff {
    ticks: u8,
}
impl Buff {
    pub fn bleed() -> Self {
        Self::Bleed( BleedDebuff { ticks: 20 } )
    }
}
impl BleedDebuff {
    pub const ICON: BuffIcon = BuffIcon::Bleed;
    
    fn tick(buffs: &mut Vec<Self>, owner: &mut Combatant) {        
        // only tick the first bleed debuff
        if let Some(b) = buffs.first_mut() {
            if b.tick_single(owner) {
                buffs.remove(0);
            }
        }
    }

    fn tick_single(&mut self, owner: &mut Combatant) -> bool {
        let stats = owner.stats();
        let tick_damage = stats.max_health / 100.0;
        let pre_res_damage = Elemental { bleed:tick_damage, fracture: 0., madness: 0., void: 0. };
        let pos_res_damage = hit::mitigation(pre_res_damage, stats.resistances);
        owner.damage(pos_res_damage.sum());

        self.ticks -= 1;
        self.ticks == 0
    }

    pub fn tooltip(buffs: &Vec<Self>, ui: &mut Ui) {
        if buffs.is_empty() {
            return;
        }
        let ticks = buffs.iter().map(|b| b.ticks).sum::<u8>() as f32;
        ui.horizontal_wrapped(|ui| {
            ui.add(Self::ICON.image());
            ui.label(format!("Take 10% of your max heath as %bleed damage every second for {} seconds", ticks/10.)); 
        });
    }
}

#[derive(Debug)]
pub struct VulnerableDebuff {
}
impl Buff {
    pub fn vulnerable() -> Self {
        Self::Vulnerable( VulnerableDebuff { } )
    }
}
impl VulnerableDebuff {
    pub const ICON: BuffIcon = BuffIcon::Vulnerable;

    pub fn apply_pre_getting_hit(&self, attack: &mut AttackPreHit) {
        attack.damage_mult = attack.damage_mult * 1.3;
    }

    pub fn tooltip(buffs: &Vec<Self>, ui: &mut Ui) {
        if buffs.is_empty() {
            return;
        }
        let mult: f32 = buffs.iter().map(|_| 1.3).product();
        ui.horizontal_wrapped(|ui| {
            ui.add(Self::ICON.image());
            ui.label(format!("The next attack against you does {}% more damage", mult*100.));
        });
    }
}

#[derive(Debug)]
pub struct ConfusedDebuff {
}
impl Buff {
    pub fn confused() -> Self {
        Self::Confused( ConfusedDebuff { } )
    }
}
impl ConfusedDebuff {
    pub const ICON: BuffIcon = BuffIcon::Confused;

    fn apply_post_hit(&self, attack: &mut AttackPostHit, _skill: &Skill, _user: &Combatant, _target: &Combatant, _hit: &Hit) {
        attack.self_hit = true;
    }

    pub fn tooltip(buffs: &Vec<Self>, ui: &mut Ui) {
        if buffs.is_empty() {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.add(Self::ICON.image());
            if buffs.len() == 1 {
                ui.label("Your next attack will also damage yourself");
            } else {
                ui.label(format!("Your next {} attacks will also damage yourself", buffs.len()));
            }
        });
    }
}

#[derive(Debug)]
pub struct EchoDebuff {
    tick_damage: Elemental<f32>,
    ticks: u8,
}
impl Buff {
    pub fn echo(damage: Elemental<f32>) -> Self {
        Self::Echo( EchoDebuff { tick_damage: damage / 100.0, ticks: 100 } )
    }
}
impl EchoDebuff {
    pub const ICON: BuffIcon = BuffIcon::Echo;
    
    fn tick(buffs: &mut Vec<Self>, owner: &mut Combatant) {
        buffs.retain_mut(|b| b.tick_single(owner));
    }

    fn tick_single(&mut self, owner: &mut Combatant) -> bool {
        let stats = owner.stats();
        let pos_res_damage = hit::mitigation(self.tick_damage, stats.resistances);
        owner.damage(pos_res_damage.sum());

        self.ticks -= 1;
        self.ticks > 0
    }

    pub fn tooltip(buffs: &Vec<Self>, ui: &mut Ui) {
        for b in buffs {
            ui.horizontal_wrapped(|ui| {
                ui.add(Self::ICON.image());
                ui.label(format!("Take {} damage over 10 seconds", b.tick_damage.sum() * 100.0));
            });
        }
    }
}

#[derive(Debug)]
pub struct LifelinkDebuff {
}
impl Buff {
    pub fn lifelink() -> Self {
        Self::Lifelink( LifelinkDebuff { } )
    }
}
impl LifelinkDebuff {
    pub const ICON: BuffIcon = BuffIcon::Lifelink;

    pub fn apply_post_getting_hit(&self, attack: &mut AttackPostHit) {
        attack.life_steal += 0.2;
    }

    pub fn tooltip(buffs: &Vec<Self>, ui: &mut Ui) {
        if buffs.is_empty() {
            return;
        }
        let sum: f32 = buffs.iter().map(|_| 0.2).sum();
        ui.horizontal_wrapped(|ui| {
            ui.add(Self::ICON.image());
            ui.label(format!("The next attack against you has an additional {}% life steal", sum*100.));
        });
    }
}

#[derive(Debug)]
pub struct IncapacitatedDebuff {
    ticks: u8,
}
impl Buff {
    pub fn incapacitated() -> Self {
        Self::Incapacitated( IncapacitatedDebuff { ticks: 20 } )
    }
}
impl IncapacitatedDebuff {    
    pub const ICON: BuffIcon = BuffIcon::Incapacitated;
    
    fn apply_to_char(&self, char: &mut CharStats) {
        char.tick_rate += 1;
    }

    fn tick(buffs: &mut Vec<Self>) {        
        // only tick the first incapacitated debuff
        if let Some(b) = buffs.first_mut() {
            b.ticks -= 1;
            if b.ticks == 0 {
                buffs.remove(0);
            }
        }
    }

    pub fn tooltip(buffs: &Vec<Self>, ui: &mut Ui) {
        if buffs.is_empty() {
            return;
        }
        let ticks = buffs.iter().map(|b| b.ticks).sum::<u8>() as f32;
        ui.horizontal_wrapped(|ui| {
            ui.add(Self::ICON.image());
            ui.label(format!("For {} seconds, cooldowns recover half as fast", ticks/10.)); 
        });
    }
}

#[derive(Debug)]
pub struct DazedDebuff {
}
impl Buff {
    pub fn dazed() -> Self {
        Self::Dazed( DazedDebuff { } )
    }
}
impl DazedDebuff {
    pub const ICON: BuffIcon = BuffIcon::Dazed;
    
    fn apply_pre_hit(&self, attack: &mut AttackPreHit, _skill: &Skill, _user: &Combatant, _target: &Combatant) {
        attack.damage_mult = attack.damage_mult * 0.5;
    }    

    pub fn tooltip(buffs: &Vec<Self>, ui: &mut Ui) {
        if buffs.is_empty() {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            ui.add(Self::ICON.image());
            if buffs.len() == 1 {
                ui.label("Your next attack will deal 50% damage");
            } else {
                ui.label(format!("Your next {} attacks will deal 50% damage", buffs.len()));
            }
        });
    }
}

#[derive(Debug)]
pub struct SoullinkDebuff {
}
impl Buff {
    pub fn soullink() -> Self {
        Self::Soullink( SoullinkDebuff { } )
    }
}
impl SoullinkDebuff {
    pub const ICON: BuffIcon = BuffIcon::Soullink;

    pub fn apply_post_getting_hit(&self, attack: &mut AttackPostHit) {
        attack.shield_steal += 0.25;
    }

    pub fn tooltip(buffs: &Vec<Self>, ui: &mut Ui) {
        if buffs.is_empty() {
            return;
        }
        let sum: f32 = buffs.iter().map(|_| 0.25).sum();
        ui.horizontal_wrapped(|ui| {
            ui.add(Self::ICON.image());
            ui.label(format!("The next attack against you has an additional {}% shield steal", sum*100.));
        });
    }
}


#[derive(PartialEq, Eq, Hash)]
pub enum BuffIcon {
    DmgUp,
    ResUp,
    ResDown,

    Bleed,
    Vulnerable,
    Confused,
    Echo,

    Lifelink,
    Incapacitated,
    Dazed,
    Soullink,
}
impl BuffIcon {    
    pub const SIZE: Vec2 = vec2(16., 16.);
    pub fn image(&self) -> Image<'_> {
        use BuffIcon::*;
        let source = match self {
            DmgUp   => include_image!("../../assets/combat/dmg_up.png"),
            ResUp   => include_image!("../../assets/combat/res_up.png"),
            ResDown => include_image!("../../assets/combat/res_down.png"),

            Bleed      => include_image!("../../assets/combat/bleed.png"),
            Vulnerable => include_image!("../../assets/combat/vulnerable.png"),
            Confused   => include_image!("../../assets/combat/confused.png"),
            Echo       => include_image!("../../assets/combat/echo.png"),

            Lifelink      => include_image!("../../assets/combat/lifelink.png"),
            Incapacitated => include_image!("../../assets/combat/incapacitated.png"),
            Dazed         => include_image!("../../assets/combat/dazed.png"),
            Soullink      => include_image!("../../assets/combat/soullink.png"),
        };
        Image::new(source).fit_to_exact_size(Self::SIZE)
    }
}