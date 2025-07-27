use crate::prelude::*;

use crate::combat::buff::Buffs;
use crate::combat::combatant::{CharStats, Combatant};
use crate::panels::animation::{Animation, AnimationPlayer};

pub fn show_char(ui: &mut Ui, char: &Combatant, char_data: &mut CombatantData, animation_override: Option<Animation>, frame: usize, rect: Rect, align: Align) {
    let char_stats = char.stats();
    let response = ui
        .allocate_new_ui(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::top_down(egui::Align::LEFT)),
            |ui| {
                ui.spacing_mut().item_spacing = vec2(0., 0.);

                // TODO make sure buffs don't push the rest "down", so either always reserve the same space on top for them, or draw them over the sprite
                // I think I can leave 16 px above for the first row, and the second overlaps? Also double check the size I alloc above I guess.
                show_buffs(ui, &char.buffs);
                show_sprite(ui, char, char_data, animation_override, frame, align);
                show_health_bar(ui, char, &char_stats);

                if char.kind.is_enemy() {
                    show_skills_bars(ui, char);
                } else {
                    show_skills_icons(ui, char);
                }

                for text in &char_data.floating {
                    let pos = rect.left_top()
                        + vec2(
                            (text.slot % 2) as f32 * 32. + 16.,
                            (text.slot / 2) as f32 * 25.0 - 5.,
                        )
                        + text.offset;
                    ui.put(Rect::from_pos(pos), Label::new(text.text.clone()).extend());
                }
            },
        )
        .response;

    response.on_hover_ui_at_pointer(|ui| show_tooltip(ui, char, &char_stats));
}

fn show_buffs(ui: &mut Ui, buffs: &Buffs) {
    let mut ui = ui.new_child(UiBuilder::new());
    ui.horizontal_wrapped(|ui| {
        // what sorting order do we want?
        buffs.icons().take(8).for_each(|icon| {
            ui.add(icon.image());
        });
    });
}

fn show_sprite(ui: &mut Ui, char: &Combatant, char_data: &mut CombatantData, animation_override: Option<Animation>, frame: usize, align: Align) {
    let (sprite_size, sprite, ) = if let Some(animation) = animation_override {
        (animation.sprite_size(), animation.frame(frame % animation.len))
    } else if let Some(animation_player) = &mut char_data.animation {
        (animation_player.sprite_size(), animation_player.frame())
    } else {
        let animation = char.idle_animation();
        (animation.sprite_size(), animation.frame((frame + char_data.idle_animation_offset as usize) % animation.len))
    };

    let align = if sprite_size.x >= 64. { align } else { Align::Center };

    let mut sprite_ui = ui.new_child(UiBuilder::new().layout(Layout::top_down(align)));
    sprite_ui.add(sprite);
    ui.add_space(sprite_size.y);
}

fn show_health_bar(ui: &mut Ui, char: &Combatant, char_stats: &CharStats) {
    let percent_health = char.health / char_stats.max_health;
    let percent_wounds = (char_stats.max_health - char.wounds) / char_stats.max_health;
    let percent_shield = char.shield / char_stats.max_health;

    let (mut allocated_rec, _) = ui.allocate_exact_size(vec2(64., 18.), Sense::hover());
    draw_heath_bar_part(ui, allocated_rec, percent_shield, Color32::LIGHT_GRAY);

    allocated_rec = allocated_rec.shrink2(vec2(1., 2.));
    draw_heath_bar_part(ui, allocated_rec, 1., Color32::BLACK);
    draw_heath_bar_part(ui, allocated_rec, percent_wounds, Color32::DARK_RED);
    draw_heath_bar_part(ui, allocated_rec, percent_health, Color32::RED);
}
fn draw_heath_bar_part(ui: &mut Ui, full_rect: Rect, percent: f32, color: Color32) {    
    let size = vec2(full_rect.width() * percent, full_rect.height());
    let rect = Rect::from_min_size(full_rect.min, size);
    ui.painter().rect_filled(rect, CornerRadius::ZERO, color);
}

fn show_skills_icons(ui: &mut Ui, char: &Combatant) {
    ui.horizontal_wrapped(|ui| {
        for skill in &char.skills {
            let (rect, _) = ui.allocate_exact_size(vec2(32., 32.), Sense::empty());

            let percent_cd = 1. - skill.cd as f32 / skill.cooldown() as f32;
            let filled = (1. - percent_cd) * rect.height();
            let inner_rect = rect.with_min_y(rect.min.y + filled);

            ui.painter()
                .rect_filled(inner_rect, CornerRadius::ZERO, Color32::DARK_GREEN);
            ui.put(rect, skill.image().fit_to_exact_size(vec2(32., 32.)));
        }
    });
}
fn show_skills_bars(ui: &mut Ui, char: &Combatant) {
    for skill in &char.skills {
        let percent_cd = 1. - skill.cd as f32 / skill.cooldown() as f32;
        let (allocated_rec, _) = ui.allocate_exact_size(vec2(64., 6.), Sense::hover());

        let rect = Rect::from_min_size(
            allocated_rec.min,
            vec2(allocated_rec.width() * percent_cd, allocated_rec.height()),
        );
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, Color32::DARK_GREEN);
    }
}

fn show_tooltip(ui: &mut Ui, char: &Combatant, char_stats: &CharStats) {
    // TODO use layout job
    ui.set_min_width(200.);

    let health = if char.shield > 0. {
        format!(
            "HP: {:.0}+{:.0} / {:.0}",
            char.health, char.shield, char_stats.max_health
        )
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
        "Heal Power: {}%, Shield Power: {}%",
        char_stats.heal_power * 100.,
        char_stats.shield_power * 100.
    ));

    ui.separator();

    // buffs
    char.buffs.tooltip(ui);
}

#[apply(Default)]
pub struct CombatantData {
    animation: Option<AnimationPlayer>,
    idle_animation_offset: u8,
    floating: Vec<FloatingText>,
}
#[derive(Debug, Clone)]
struct FloatingText {
    text: RichText,
    slot: u32, // TODO see slots below
    tick: u8,
    offset: Vec2,
}

impl CombatantData {
    pub fn new(idle_animation_offset: u8) -> Self {
        Self { animation: None, idle_animation_offset, floating: Vec::new() }
    }
    pub fn tick(&mut self) {
        if let Some(animation_player) = &mut self.animation {
            if !animation_player.next_frame() {
                self.animation = None
            }
        }

        self.floating.iter_mut().for_each(|t| t.tick += 1);
        self.floating.retain(|t| t.tick < 8);
    }
    pub fn add_animation(&mut self, animation: Animation) {
        self.animation = Some(AnimationPlayer::new(animation));
    }
    pub fn add_text(&mut self, text: RichText) {
        let mut rng = rand::rng();
        let used_slots: Vec<u32> = self.floating.iter().map(|f| f.slot).collect();
        let open_slots: Vec<u32> = [0, 1, 2, 3, 4, 5]
            .into_iter()
            .filter(|s| !used_slots.contains(s))
            .collect();
        let slot = match open_slots.len() {
            0 => {
                let oldest = self
                    .floating
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, f)| f.tick)
                    .unwrap()
                    .0;
                self.floating.swap_remove(oldest).slot
            }
            1 => open_slots[0],
            _ => *open_slots.pick(&mut rng),
        };

        let offset = vec2(rng.random_range(-5.0..5.0), rng.random_range(-5.0..5.0));
        self.floating.push(FloatingText {
            text,
            slot,
            tick: 0,
            offset,
        });
    }
}