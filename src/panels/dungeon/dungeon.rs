use crate::combat::combatant::CombatantKind;
use crate::combat::skill::skill::SkillStats;
use crate::dungeon::dungeon_data::DungeonTick;
use crate::dungeon::encounter::EncounterDifficulty;
use crate::dungeon::floor::Floor;
use crate::equipment::wardrobe::Wardrobe;
use crate::panels::animation::Animation;
use crate::panels::dungeon::combatant::{self, CombatantData};
use crate::panels::rewards::{self, RewardsWindow};
use crate::prelude::*;

use crate::{
    combat::battle::Battle,
    dungeon::dungeon_data::DungeonData,
    timekeeper::FrameInfo,
};

#[apply(Default)]
pub struct DungeonPanel {
    battle: BattleData,
}

#[apply(Default)]
struct BattleData {
    fighter: CombatantData,
    enemies: Vec<CombatantData>,
}

impl DungeonPanel {
    pub fn tick(
        &mut self,
        dungeon: &DungeonData,
        frame_info: &FrameInfo,
        just_finished_loading: bool,
    ) {
        if !dungeon.cur.finished {
            if just_finished_loading || frame_info.catch_up.is_some() || frame_info.dungeon_tick.as_ref().is_some_and(|t| t.new_battle) {
                self.battle = BattleData::from(&dungeon.cur.floor.battle);
            }

            if frame_info.tick {
                self.battle.tick(&frame_info.dungeon_tick);
            }
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        dungeon: &mut DungeonData,
        wardrobe: &Wardrobe,
        rewards_window: &mut RewardsWindow,
        frame_info: &FrameInfo,
    ) {
        ui.horizontal(|ui| {
            ui.heading("Dungeon");
            if dungeon.rewards.len() > 0 {
                if rewards::add_chest_button(ui).clicked() {
                    rewards_window.open();
                }
                let rewards: usize = dungeon.rewards.values().map(|r| r.len()).sum();
                if rewards > 1 {
                    ui.label(format!("x{}", rewards));
                }
            }

            ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                if ui.button("Cancel Run").clicked() {
                    dungeon.cur.cancel();
                }
                if ui
                    .checkbox(&mut dungeon.auto_restart, "auto restart")
                    .clicked()
                    && dungeon.auto_restart
                    && dungeon.cur.finished
                {
                    dungeon.restart(wardrobe.equipped());
                }
            });
        });

        ui.separator();

        ScrollArea::vertical().show(ui, |ui|{
            show_floor_info(ui, &dungeon.cur.floor);
            
            ui.scope(|ui| {
                ui.style_mut().interaction.selectable_labels = false;
                let rect = ui.add(dungeon.cur.area.background.image()).rect;
                ui.set_clip_rect(rect);
                
                if dungeon.cur.finished {
                    if show_stats(ui, rect).clicked() {
                        dungeon.restart(wardrobe.equipped());
                    }
                } else {                    
                    let transition = dungeon
                        .cur
                        .floor
                        .transition
                        .map(|f| (Floor::TRANSITION_TIME - f, frame_info.anim(ui.ctx(), 30, f, easing::linear)));
                
                    show_battle(
                        ui,
                        rect,
                        &dungeon.cur.floor.battle,
                        &mut self.battle,
                        transition,
                    );
                };
            
                #[cfg(debug_assertions)]
                ui.put(
                    Rect::from_min_max(
                        pos2(rect.right() - 75., rect.top()),
                        pos2(rect.right(), rect.top() + 20.),
                    ),
                    Label::new(format!("{:.0} ms", frame_info.delay * 1000.)),
                );
                ui.force_set_min_rect(rect);
            });
        });
    }
}

fn show_battle(
    ui: &mut Ui,
    rect: Rect,
    battle: &Battle,
    battle_data: &mut BattleData,
    transition: Option<(u32, f32)>,
) {
    let center = rect.center();
    let frame = transition.map_or(battle.tick as usize, |(t,_)| t as usize);
    combatant::show_char(
        ui,
        &battle.fighter,
        &mut battle_data.fighter,
        transition.as_ref().map(|_| Animation::FighterWalk),
        frame,
        Rect::from_center_size(center + FIGHTER_OFFSET, vec2(64., 120.)),
        Align::LEFT,
    );

    let transition_offset = if let Some((_, progress)) = transition {
        vec2((1. - progress) * 250., 0.)
    } else {
        vec2(0., 0.)
    };

    for ((offset, enemy), enemy_data) in ENEMY_OFFSETS[battle.enemies.len()]
        .iter()
        .zip(battle.enemies.iter())
        .zip(battle_data.enemies.iter_mut())
    {
        if !enemy.alive() {
            continue;
        }
        combatant::show_char(
            ui,
            enemy,
            enemy_data,
            None,
            frame,
            Rect::from_center_size(center + *offset + transition_offset, vec2(64., 100.)),
            Align::RIGHT,
        );
    }
}

const FIGHTER_OFFSET: Vec2 = vec2(-Background::SIZE.x / 4., 40.);
const ENEMY_OFFSETS: [&[Vec2]; 7] = [
    &[],
    &[vec2(Background::SIZE.x / 4., 32.)],
    &[
        vec2(Background::SIZE.x / 4., -32.),
        vec2(Background::SIZE.x / 4., 90.),
    ],
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
    &[
        vec2(-75. + Background::SIZE.x / 4., -5.),
        vec2(-75. + Background::SIZE.x / 4., 80.),
        vec2(5. + Background::SIZE.x / 4., -35.),
        vec2(5. + Background::SIZE.x / 4., 110.),
        vec2(85. + Background::SIZE.x / 4., 40.),
    ],
    &[
        vec2(-75. + Background::SIZE.x / 4., -5.),
        vec2(-75. + Background::SIZE.x / 4., 80.),
        vec2(5. + Background::SIZE.x / 4., -35.),
        vec2(5. + Background::SIZE.x / 4., 110.),
        vec2(85. + Background::SIZE.x / 4., -5.),
        vec2(85. + Background::SIZE.x / 4., 80.),
    ],
];


fn show_stats(ui: &mut Ui, rect: Rect) -> Response {
    let rect = Rect::from_center_size(rect.center(), vec2(120., 20.));
    ui.put(rect, Button::new("Start Dungeon"))
}

fn show_floor_info(ui: &mut Ui, floor: &Floor) {
    ui.horizontal(|ui| {
        ui.label(RichText::from(format!("Floor {}:", floor.depth)).size(18.));
        for (i, encounter) in floor.encounters.iter().enumerate() {
            let desired_size = Vec2::splat(ui.spacing().icon_width);
            let (rect, _) = ui.allocate_exact_size(desired_size, Sense::hover());
            
            let color = match encounter.difficulty {
                EncounterDifficulty::Easy => Color32::GOLD,
                EncounterDifficulty::Medium => Color32::ORANGE,
                EncounterDifficulty::Hard | EncounterDifficulty::Boss => Color32::RED,
            };

            let (small_icon_rect, big_icon_rect) = ui.spacing().icon_rectangles(rect);
            // try this instead ui.painter().rect_stroke(rect, corner_radius, stroke, stroke_kind)
            ui.painter().add(epaint::RectShape::new(
                big_icon_rect,
                CornerRadius::same(2),
                Color32::TRANSPARENT,
                Stroke::new(1.5, color),
                epaint::StrokeKind::Outside,
            ));

            if i < floor.battle_counter as usize - 1 {
                ui.painter().add(Shape::line(
                    vec![
                        pos2(small_icon_rect.left(), small_icon_rect.center().y),
                        pos2(small_icon_rect.center().x, small_icon_rect.bottom()),
                        pos2(small_icon_rect.right(), small_icon_rect.top()),
                    ],
                    Stroke::new(1.5, Color32::WHITE),
                ));
            }
        }
    });
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
            Forest => include_image!("../../../assets/backgrounds/battleback1.png"),
            RedForest => include_image!("../../../assets/backgrounds/battleback7.png"),
            Snow => include_image!("../../../assets/backgrounds/battleback2.png"),
            Desert => include_image!("../../../assets/backgrounds/battleback3.png"),
            Cave => include_image!("../../../assets/backgrounds/battleback5.png"),
            Dungeon => include_image!("../../../assets/backgrounds/battleback8.png"),
        };
        Image::new(source).fit_to_exact_size(Self::SIZE)
    }
}

impl BattleData {
    fn from(battle: &Battle) -> Self {
        let mut rng = rand::rng();
        Self {
            fighter: CombatantData::new(rng.random()),
            enemies: battle
                .enemies
                .iter()
                .map(|_| CombatantData::new(rng.random()))
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
                match &attack.attacker {
                    CombatantKind::Fighter => self.fighter.add_animation(Animation::FighterAttack),
                    CombatantKind::Enemy(i, enemy_kind) => self.enemies[*i as usize].add_animation(enemy_kind.attack_animation()),
                }
                // play attack animation of attacker

                for hit in &attack.hits {
                    let text = RichText::from(format!("{:.0}", hit.hit.post_res_dmg.sum()))
                        .color(Color32::RED);
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
