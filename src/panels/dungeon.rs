use crate::combat::buff::Buffs;
use crate::combat::combatant::{CharStats, CombatantKind};
use crate::combat::skill::skill::SkillStats;
use crate::dungeon::dungeon::DungeonTick;
use crate::equipment::wardrobe::Wardrobe;
use crate::prelude::*;

use crate::{
    combat::{battle::Battle, combatant::Combatant},
    dungeon::dungeon::DungeonData,
    timekeeper::FrameInfo,
};

use super::{
    animation::Animation,
    rewards::{add_chest_button, RewardsWindow},
};

#[apply(Default)]
pub struct DungeonPanel {
    battle: BattleData,
}

#[apply(Default)]
struct BattleData {
    depth: u16,
    fighter: CombatantData,

    enemies: Vec<CombatantData>,
}
#[apply(Default)]
struct CombatantData {
    animation: AnimationState,
    floating: Vec<FloatingText>,
}

impl DungeonPanel {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        dungeon: &mut DungeonData,
        wardrobe: &Wardrobe,
        rewards_window: &mut RewardsWindow,
        frame_info: FrameInfo,
    ) {
        ui.horizontal(|ui| {
            ui.heading("Dungeon");
            if dungeon.rewards.len() > 0 {
                if add_chest_button(ui).clicked() {
                    rewards_window.open();
                }
                if dungeon.rewards.len() > 1 {
                    ui.label(format!("x{}", dungeon.rewards.len()));
                }
            }

            ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                if ui.button("Cancel Run").clicked() {
                    dungeon.cur.battle.fighter.health = 0.;
                }
                if ui.checkbox(&mut dungeon.auto_restart, "auto restart").clicked()
                    && dungeon.auto_restart
                    && dungeon.cur.finished
                {
                    dungeon.restart(wardrobe.equipped());
                }
            });
        });

        ui.separator();

        ui.scope(|ui| {
            ui.style_mut().interaction.selectable_labels = false;
            let rect = ui.add(dungeon.cur.area.background.image()).rect;
            ui.set_clip_rect(rect);

            if dungeon.cur.finished {
                if show_stats(ui, rect).clicked() {
                    dungeon.restart(wardrobe.equipped());
                    self.battle = BattleData::from(&dungeon.cur.battle, 0);
                }
            } else {
                if frame_info.tick {
                    if dungeon.cur.depth != self.battle.depth {
                        self.battle = BattleData::from(&dungeon.cur.battle, dungeon.cur.depth);
                    } else {
                        self.battle.tick(&frame_info.dungeon_tick);
                    }
                }

                let transition = dungeon.cur.transition.map(|f| frame_info.anim(ui.ctx(), 30, f, easing::linear));
                if transition.is_none() {
                    self.battle.fighter.set_animation(Animation::FighterAttack);
                }

                show_battle(ui, rect, &dungeon.cur.battle, &self.battle, transition);
            };

            ui.put(
                Rect::from_min_max(pos2(rect.left(), rect.top()), pos2(rect.left() + 75., rect.top() + 20.)),
                Label::new(RichText::from(format!("Depth: {}", dungeon.cur.depth)).color(Color32::DARK_RED)),
            );

            // only show this in debug?
            ui.put(
                Rect::from_min_max(pos2(rect.right() - 75., rect.top()), pos2(rect.right(), rect.top() + 20.)),
                Label::new(format!("{:.0} ms", frame_info.delay * 1000.)),
            );
            ui.force_set_min_rect(rect);
        });
    }
}

fn show_battle(ui: &mut Ui, rect: Rect, battle: &Battle, battle_data: &BattleData, transition: Option<f32>) {
    let center = rect.center();
    // TODO show walking animation if transition
    show_char(ui, &battle.fighter, &battle_data.fighter, Rect::from_center_size(center + FIGHTER_OFFSET, vec2(64., 120.)));

    let transition_offset = if let Some(progress) = transition {
        vec2((1. - progress) * 250., 0.)
    } else {
        vec2(0., 0.)
    };

    for ((offset, enemy), enemy_data) in ENEMY_OFFSETS[battle.enemies.len()]
        .iter()
        .zip(battle.enemies.iter())
        .zip(battle_data.enemies.iter())
    {
        if !enemy.alive() {
            continue;
        }
        show_char(ui, enemy, enemy_data, Rect::from_center_size(center + *offset + transition_offset, vec2(64., 100.)));
    }
}

const FIGHTER_OFFSET: Vec2 = vec2(-Background::SIZE.x / 4., 40.);
const ENEMY_OFFSETS: [&[Vec2]; 5] = [
    &[],
    &[vec2(Background::SIZE.x / 4., 32.)],
    &[vec2(Background::SIZE.x / 4., -32.), vec2(Background::SIZE.x / 4., 90.)],
    &[
        vec2(-30. + Background::SIZE.x / 4., -15.),
        vec2(-30. + Background::SIZE.x / 4., 90.),
        vec2(70. + Background::SIZE.x / 4., 40.),
    ],
    &[
        vec2(-30. + Background::SIZE.x / 4., -15.),
        vec2(-30. + Background::SIZE.x / 4., 90.),
        vec2(70. + Background::SIZE.x / 4., -15.),
        vec2(70. + Background::SIZE.x / 4., 90.),
    ],
];

fn show_char(ui: &mut Ui, char: &Combatant, char_data: &CombatantData, rect: Rect) {
    let char_stats = char.stats();
    let response = ui.allocate_new_ui(UiBuilder::new().max_rect(rect).layout(Layout::top_down(egui::Align::LEFT)), |ui| {
        ui.spacing_mut().item_spacing = vec2(0., 0.);

        show_buffs(ui, &char.buffs); // TODO make sure buffs don't push the rest "down", so either always reserve the same space on top for them, or draw them over the sprite
                                        // I think I can leave 16 px above for the first row, and the second overlaps? Also double check the size I alloc above I guess.
        show_sprite(ui, char_data.animation.frame());
        show_health_bar(ui, char, &char_stats);

        if char.kind.is_enemy() {
            show_skills_bars(ui, char);
        } else {
            show_skills_icons(ui, char);
        }

        for text in &char_data.floating {
            let pos = rect.left_top() + vec2((text.slot % 2) as f32 * 32. + 16., (text.slot / 2) as f32 * 25.0 - 5.) + text.offset;
            ui.put(Rect::from_pos(pos), Label::new(text.text.clone()).extend());
        }
    }).response;

    response.on_hover_ui_at_pointer(|ui| show_tooltip(ui, char, &char_stats));
}

fn show_buffs(ui: &mut Ui, buffs: &Buffs) {
    let mut ui = ui.new_child(UiBuilder::new());
    ui.horizontal_wrapped(|ui| {
        // what sorting order do we want?
        buffs.icons().take(8).for_each(|icon| { ui.add(icon.image()); });
    });
}

fn show_sprite(ui: &mut Ui, sprite: Image<'_>) {
    let mut sprite_ui = ui.new_child(UiBuilder::new());
    sprite_ui.add(sprite);
    let mut min_rect = sprite_ui.min_rect();
    if min_rect.width() > 64. {
        min_rect.set_width(64.);
    }
    ui.advance_cursor_after_rect(min_rect);
}

fn show_health_bar(ui: &mut Ui, char: &Combatant, char_stats: &CharStats) {
    let percent_health = char.health / char_stats.max_health;
    let percent_shield = (char.shield / char_stats.max_health).at_most(1.);

    let (allocated_rec, _) = ui.allocate_exact_size(vec2(64., 16.), Sense::hover());

    if percent_health + percent_shield > 1. {
        ui.painter().rect_filled(allocated_rec, CornerRadius::ZERO, Color32::RED);
        let shield_rect = Rect::from_min_max(
            allocated_rec.min + vec2(allocated_rec.width() * (1. - percent_shield), 0.),
            allocated_rec.max,
        );
        ui.painter().rect_filled(shield_rect, CornerRadius::ZERO, Color32::LIGHT_GRAY);
    } else {
        ui.painter().rect_filled(allocated_rec, CornerRadius::ZERO, Color32::BLACK);
        let shield_rect = Rect::from_min_size(
            allocated_rec.min,
            vec2(
                allocated_rec.width() * (percent_shield + percent_health),
                allocated_rec.height(),
            ),
        );
        ui.painter().rect_filled(shield_rect, CornerRadius::ZERO, Color32::LIGHT_GRAY);
        let health_rect = Rect::from_min_size(
            allocated_rec.min,
            vec2(
                allocated_rec.width() * percent_health,
                allocated_rec.height(),
            ),
        );
        ui.painter().rect_filled(health_rect, CornerRadius::ZERO, Color32::RED);
    }
}

fn show_skills_icons(ui: &mut Ui, char: &Combatant) {
    ui.horizontal_wrapped(|ui| {
        for skill in &char.skills {
            let (rect, _) = ui.allocate_exact_size(vec2(32., 32.), Sense::empty());

            let percent_cd = 1. - skill.cd as f32 / skill.cooldown() as f32;
            let filled = (1. - percent_cd) * rect.height();
            let inner_rect = rect.with_min_y(rect.min.y + filled);

            ui.painter().rect_filled(inner_rect, CornerRadius::ZERO, Color32::DARK_GREEN);
            ui.put(rect, skill.image().fit_to_exact_size(vec2(32., 32.)));
        }
    });
}
fn show_skills_bars(ui: &mut Ui, char: &Combatant) {
    for skill in &char.skills {
        let percent_cd = 1. - skill.cd as f32 / skill.cooldown() as f32;
        let (allocated_rec, _) = ui.allocate_exact_size(vec2(64., 6.), Sense::hover());

        let rect = Rect::from_min_size(allocated_rec.min, vec2(allocated_rec.width() * percent_cd, allocated_rec.height()));
        ui.painter().rect_filled(rect, CornerRadius::ZERO, Color32::DARK_GREEN);
    }
}

fn show_stats(ui: &mut Ui, rect: Rect) -> Response {
    let rect = Rect::from_center_size(rect.center(), vec2(120., 20.));
    ui.put(rect, Button::new("Start Dungeon"))
}

fn show_tooltip(ui: &mut Ui, char: &Combatant, char_stats: &CharStats) {
    // TODO use layout job
    ui.set_min_width(200.);

    let health = if char.shield > 0. {
        format!("HP: {:.0}+{:.0} / {:.0}", char.health, char.shield, char_stats.max_health)
    } else {
        format!("HP:  {:.0} / {:.0}", char.health, char_stats.max_health)
    };
    ui.label(health);
    ui.label(format!(
        "Resistances: {}/{}/{}/{}",
        char_stats.resistances.bleed,
        char_stats.resistances.fracture,
        char_stats.resistances.madness,
        char_stats.resistances.void
    ));
    ui.label(format!(
        "Heal Power: {}%, Shield Power: {}%", char_stats.heal_power * 100., char_stats.shield_power * 100.
    ));

    ui.separator();

    // buffs
    char.buffs.tooltip(ui);
}

#[apply(UnitEnum)]
#[derive(Default)]
pub enum Background {
    Forest,
    RedForest,
    Snow,
    Desert,
    Cave,
    #[default]
    Dungeon,
}
impl Background {
    pub const SIZE: Vec2 = vec2(/* 1104./2.*/ 552., /*621./2.*/ 310.5);
    pub fn image(&self) -> Image<'_> {
        use Background::*;
        let source = match *self {
            Forest => include_image!("../../assets/backgrounds/battleback1.png"),
            RedForest => include_image!("../../assets/backgrounds/battleback7.png"),
            Snow => include_image!("../../assets/backgrounds/battleback2.png"),
            Desert => include_image!("../../assets/backgrounds/battleback3.png"),
            Cave => include_image!("../../assets/backgrounds/battleback5.png"),
            Dungeon => include_image!("../../assets/backgrounds/battleback8.png"),
        };
        Image::new(source).fit_to_exact_size(Self::SIZE)
    }
}

#[apply(Default)]
struct AnimationState {
    // TODO better name
    animation: Animation,
    frame: usize,
    // something that identifies the animation? Or at least a flag if it is the idle animation or not
    // and then it will need more stuff when we have actual logic
    // for melee chars, we need like a vec of animations playing at not normal pos
}
#[derive(Debug, Clone)]
struct FloatingText {
    text: RichText,
    slot: u32, // TODO see slots below
    tick: u8,
    offset: Vec2,
}
impl BattleData {
    fn from(battle: &Battle, depth: u16) -> Self {
        Self {
            depth,
            fighter: CombatantData {
                animation: AnimationState::from(Animation::FighterWalk),
                floating: Vec::new(),
            },
            enemies: battle
                .enemies
                .iter()
                .map(|c| CombatantData {
                    animation: AnimationState::from(c.idle_animation()),
                    floating: Vec::new(),
                })
                .collect(),
        }
    }
    fn tick(&mut self, dungeon_tick: &Option<DungeonTick>) {
        self.fighter.tick();
        self.enemies.iter_mut().for_each(|e| e.tick());

        if let Some(tick) = dungeon_tick {
            tick.skills.iter().for_each(|s| self.add_skill(s));
        }
    }
    fn add_skill(&mut self, skill: &SkillStats) {
        match skill {
            SkillStats::Attack(_, attack) => {
                for hit in &attack.hits {
                    let text = RichText::from(format!("{:.0}", hit.hit.post_res_dmg.sum())).color(Color32::RED);
                    match &hit.target {
                        CombatantKind::Fighter => self.fighter.add_text(text),
                        CombatantKind::Enemy(i, _) => self.enemies[*i as usize].add_text(text),
                    }
                    hit.responses.iter().for_each(|r| self.add_skill(r));
                }
            }
            SkillStats::Defend(_, def_stats) => {
                let total = def_stats.healed + def_stats.shielded;
                let color = if def_stats.healed > 0. {
                    Color32::GREEN
                } else {
                    Color32::LIGHT_GRAY
                };
                let text = RichText::from(format!("{:.0}", total)).color(color);
                match &def_stats.defender {
                    CombatantKind::Fighter => self.fighter.add_text(text),
                    CombatantKind::Enemy(i, _) => self.enemies[*i as usize].add_text(text),
                }
            }
        }
    }
}
impl CombatantData {
    fn tick(&mut self) {
        self.animation.tick();
        self.floating.iter_mut().for_each(|t| t.tick += 1);
        self.floating.retain(|t| t.tick < 8);
    }
    fn set_animation(&mut self, animation: Animation) {
        if self.animation.animation != animation {
            self.animation = AnimationState::from(animation);
        }
    }
    fn add_text(&mut self, text: RichText) {
        let mut rng = rand::rng();
        let used_slots: Vec<u32> = self.floating.iter().map(|f| f.slot).collect();
        let open_slots: Vec<u32> = [0, 1, 2, 3, 4, 5].into_iter().filter(|s| !used_slots.contains(s)).collect();
        let slot = match open_slots.len() {
            0 => {
                let oldest = self.floating.iter().enumerate().max_by_key(|(_, f)| f.tick).unwrap().0;
                self.floating.swap_remove(oldest).slot
            }
            1 => open_slots[0],
            _ => *open_slots.choose(&mut rng).unwrap(),
        };

        let offset = vec2(rng.random_range(-5.0..5.0), rng.random_range(-5.0..5.0));
        self.floating.push(FloatingText { text, slot, tick: 0, offset });
    }
}
impl From<Animation> for AnimationState {
    fn from(value: Animation) -> Self {
        Self { animation: value, frame: 0 }
    }
}
impl AnimationState {
    fn tick(&mut self) {
        self.frame += 1;
        self.frame %= 2 * self.animation.len;
    }
    fn frame(&self) -> Image<'_> {
        self.animation.frame(self.frame / 2)
    }
}
