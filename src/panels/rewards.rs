use std::f32::consts::PI;

use crate::{
    dungeon::reward::RewardChest,
    item::{Item, ItemType},
    prelude::*,
    stash::stash::Stash,
};
use egui::{emath::inverse_lerp, epaint::RectShape};
use rand::distr::Uniform;
use rand_chacha::ChaCha12Rng;
use web_time::SystemTime;

use crate::{dungeon::dungeon::DungeonData, widgets::selectable_image::SelectableImage};

#[derive(Debug, SmartDefault)]
pub struct RewardsWindow {
    open: bool,
    opening: Option<ChestOpening>,
}

impl RewardsWindow {
    pub fn open(&mut self) {
        self.open = true;
    }
    pub fn show(&mut self, ctx: &Context, dungeon: &mut DungeonData, stash: &mut Stash) {
        // TODO test what happens when there are so many chests that it overflows the screen -> no longer visible
        // well I think I want a max height and a scroll anyways...
        Window::new("Rewards")
            .enabled(self.opening.is_none())
            .open(&mut self.open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                let dont_draw_idx = self.opening.as_ref().map(|o| o.chest_idx).unwrap_or(usize::MAX);
                for (i, reward) in dungeon.rewards.iter().enumerate() {
                    if i > 0 {
                        ui.separator();
                    }
                    ui.horizontal(|ui| {
                        if dont_draw_idx == i {
                            add_chest_button_placeholder(ui);
                        } else {
                            let response = add_chest_button(ui);
                            if response.clicked() {
                                self.opening = Some(ChestOpening {
                                    start: SystemTime::now(),
                                    chest_start_rect: response.rect,
                                    chest_idx: i,
                                    seed: [0; 32],
                                });
                                self.opening.as_mut().map(|op| rand::rng().fill_bytes(&mut op.seed));
                            }
                        }

                        if reward.items.len() == 1 {
                            ui.label("contains 1 item");
                        } else {
                            ui.label(format!("contains {} items", reward.items.len()));
                        }
                        // once we have run stats, a button here that opens the stats of the respective run could be nice
                    });
                }
            });

        if let Some(opening) = &mut self.opening {
            if opening.show(ctx, &dungeon.rewards[opening.chest_idx]) {
                let reward = dungeon.rewards.remove(opening.chest_idx);
                reward.items.into_iter().for_each(|item| stash.add(item));
                self.opening = None;
            }
        }

        if dungeon.rewards.is_empty() {
            self.open = false;
        }
    }
}

#[derive(Debug)]
struct ChestOpening {
    start: SystemTime,
    chest_start_rect: Rect,
    chest_idx: usize,
    seed: [u8; 32],
}

impl ChestOpening {
    fn target(&self, ui: &Ui) -> Rect {
        Rect::from_center_size(ui.max_rect().center(), self.chest_start_rect.size())
    }

    fn show(&self, ctx: &Context, chest: &RewardChest) -> bool {
        ctx.request_repaint();

        let mut ui = Ui::new(
            ctx.clone(),
            "ChestOpeningUi".into(),
            UiBuilder::new().layer_id(LayerId::new(Order::Foreground, "ChestOpeningLayer".into())),
        );

        let elapsed = self.start.elapsed().unwrap().as_secs_f32();
        match elapsed {
            0.0..0.5 => self.move_to_center(&mut ui, inverse_lerp(0.0..=0.5, elapsed).unwrap()),
            0.5..1.2 => self.charge(&mut ui, inverse_lerp(0.5..=1.2, elapsed).unwrap()),
            1.2..1.5 => self.idle(&mut ui),
            1.5..2.5 => self.explode(&mut ui, &chest.items, inverse_lerp(1.5..=2.5, elapsed).unwrap()),
            2.5..5.0 => self.fade(&mut ui, &chest.items, inverse_lerp(2.5..=5.0, elapsed).unwrap()),
            5.0..7.5 => self.spin(&mut ui, &chest.items, inverse_lerp(5.0..=7.5, elapsed).unwrap()),
            7.5..8.0 => self.collect(&mut ui, &chest.items, inverse_lerp(7.5..=8.0, elapsed).unwrap()),
            _ => return self.show_items(ctx, &chest.items),
        };
        return false;
    }

    fn move_to_center(&self, ui: &mut Ui, t: f32) {
        let pos = self.chest_start_rect.lerp_towards(&self.target(ui), easing::quadratic_in_out(t));
        ui.put(pos, chest_img());
    }

    fn charge(&self, ui: &mut Ui, t: f32) {
        let target = self.target(ui);

        let full_size = ui.max_rect().size().min_elem() * 0.8;
        let size = lerp(full_size..=20.0, easing::cubic_out(t));
        let color = Color32::YELLOW.gamma_multiply(t);
        ui.painter().add(circle(target.center(), size, color));

        ui.painter().extend(rays(
            self.seed,
            ui.max_rect(),
            30,
            0.33,
            0.0..=1.0,
            false,
            t,
        ));
        ui.put(target, chest_img());
    }

    fn idle(&self, ui: &mut Ui) {
        ui.put(self.target(ui), chest_img());
    }

    fn explode(&self, ui: &mut Ui, items: &Vec<Item>, t: f32) {
        let target = self.target(ui);

        let full_size = ui.max_rect().size().min_elem() * 4.0;
        let size = lerp(20.0..=full_size, t);
        ui.painter().add(circle(target.center(), size, Color32::YELLOW.gamma_multiply(0.8)));

        ui.painter().extend(rays(self.seed, ui.max_rect(), 50, 0.5, 0.0..=1.0, true, t));

        let item_max_dist = 500.0.at_most(ui.max_rect().size().min_elem()) * 0.5;
        let item_dist = lerp(0.0..=item_max_dist, easing::quadratic_out(t));
        let item_rotation = lerp(0.0..=PI, easing::quadratic_out(t));
        spin_items(ui, items, item_dist, item_rotation);

        ui.put(target, open_chest_img());
    }

    fn fade(&self, ui: &mut Ui, items: &Vec<Item>, t: f32) {
        let target = self.target(ui);

        let full_size = ui.max_rect().size().min_elem() * 4.0;
        ui.painter().add(circle(target.center(), full_size, Color32::YELLOW.gamma_multiply(lerp(0.8..=0.0, easing::quadratic_in_out(t)))));

        let item_max_dist = 500.0.at_most(ui.max_rect().size().min_elem()) * 0.5;
        let item_rotation = lerp(PI..=1.5 * PI, t);
        spin_items(ui, items, item_max_dist, item_rotation);

        ui.put(target, open_chest_img());
    }

    fn spin(&self, ui: &mut Ui, items: &Vec<Item>, t: f32) {
        let target = self.target(ui);

        let item_max_dist = 500.0.at_most(ui.max_rect().size().min_elem()) * 0.5;
        let item_rotation = lerp(1.5 * PI..=2.0 * PI, t);
        spin_items(ui, items, item_max_dist, item_rotation);

        ui.put(target, open_chest_img());
    }

    fn collect(&self, ui: &mut Ui, items: &Vec<Item>, t: f32) {
        let target = self.target(ui);

        let item_max_dist = 500.0.at_most(ui.max_rect().size().min_elem()) * 0.5;
        spin_items(ui, items, lerp(item_max_dist..=0.0, easing::quadratic_in_out(t)), 0.);

        ui.put(target, open_chest_img());
    }

    fn show_items(&self, ctx: &Context, items: &Vec<Item>) -> bool {
        let mut close = false;
        Window::new("Opened Chest")
            .title_bar(false)
            .resizable(false)
            .show(ctx, |ui| {
                for item in items {
                    ui.horizontal_top(|ui| {
                        item.show(ui);
                        ui.vertical(|ui| {
                            for modifier in &item.mods {
                                modifier.show_tooltip(ui);
                            }
                        });
                    });
                    ui.separator();
                }
                ui.vertical_centered(|ui| {
                    if ui.button("Add to Loot").clicked() {
                        close = true;
                    }
                });
            });
        return close;
    }
}

fn circle(center: Pos2, diameter: f32, color: Color32) -> RectShape {
    RectShape::new(
        Rect::from_center_size(center, vec2(diameter, diameter)),
        255,
        color,
        Stroke::NONE,
        StrokeKind::Outside,
    )
    .with_blur_width(diameter * 0.5)
}

fn spin_items(ui: &mut Ui, items: &Vec<Item>, dist: f32, rot: f32) {
    let rot_offset = 2.0 * PI / items.len() as f32;

    for (idx, item) in items.iter().enumerate() {
        let rot = rot + (idx as f32 * rot_offset);
        let vec = Vec2::from(rot.sin_cos());
        let pos = ui.max_rect().center() + (vec * dist);
        let rect = Rect::from_center_size(pos, ItemType::SIZE);
        item.show(&mut ui.new_child(UiBuilder::new().max_rect(rect)));
    }
}

fn rays(
    seed: [u8; 32],
    screen: Rect,
    amount: usize,
    duration: f32,
    spread_between: RangeInclusive<f32>,
    reverse: bool,
    t: f32,
) -> Vec<Shape> {
    let mut rng = ChaCha12Rng::from_seed(seed);

    let start_uniform =
        Uniform::new(spread_between.start(), spread_between.end() - duration).unwrap();
    let angle_uniform = Uniform::new(0., 2. * PI).unwrap();
    const COLORS: [Color32; 6] = [
        Color32::RED,
        Color32::ORANGE,
        Color32::YELLOW,
        Color32::CYAN,
        Color32::LIGHT_BLUE,
        Color32::LIGHT_GREEN,
    ];

    let mut rays = Vec::new();
    for _ in 0..amount {
        let start = start_uniform.sample(&mut rng);
        let angle = angle_uniform.sample(&mut rng);
        let color = *COLORS.choose(&mut rng).unwrap();

        let mut progress = inverse_lerp(start..=start + duration, t).unwrap();
        if progress < 0. || progress > 1. {
            continue;
        };
        if reverse {
            progress = 1. - progress
        };
        rays.push(ray(screen, angle, color, progress));
    }
    rays
}
fn ray(screen: Rect, angle: f32, color: Color32, progress: f32) -> Shape {
    let center = screen.center();
    let vec = Vec2::from(angle.sin_cos());
    let len = screen.size().max_elem() / 2.;

    let max = center + (vec * len);
    let max_extended = center + (vec * len * 1.3);

    let start = max.lerp(center, progress);
    let end = max_extended.lerp(center, progress);

    Shape::LineSegment { points: [start, end], stroke: Stroke { width: 8., color }}
}

pub fn add_chest_button(ui: &mut Ui) -> Response {
    ui.add(SelectableImage::new(false, chest_img()))
}
fn add_chest_button_placeholder(ui: &mut Ui) -> Response {
    ui.add(SelectableImage::new(false, chest_img().tint(Color32::TRANSPARENT)))
}

fn chest_img<'a>() -> Image<'a> {
    Image::new(egui::include_image!("../../assets/icons/locked-chest.png"))
        .fit_to_exact_size(vec2(32., 32.))
        .tint(Color32::GOLD)
}
fn open_chest_img<'a>() -> Image<'a> {
    Image::new(egui::include_image!(
        "../../assets/icons/open-treasure-chest.png"
    ))
    .fit_to_exact_size(vec2(32., 32.))
    .tint(Color32::GOLD)
}
