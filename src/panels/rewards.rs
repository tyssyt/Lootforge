use std::f32::consts::PI;

use crate::{
    dungeon::reward::RewardChest,
    item::{item::Item, item_type::ItemType},
    prelude::*,
    stash::stash::Stash, widgets::text_in_rect::text_in_rect,
};
use egui::{emath::inverse_lerp, epaint::RectShape};
use rand::distr::Uniform;
use rand_chacha::ChaCha12Rng;
use web_time::SystemTime;

use crate::{dungeon::dungeon_data::DungeonData, widgets::selectable_image::SelectableImage};

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
            .scroll([false, true])
            .show(ctx, |ui| {
                let mut dont_draw_idx = self.opening.as_ref().map(|o| &o.chest_idx).cloned();
                let mut first = true;
                for (&depth, rewards) in dungeon.rewards.iter() {
                    if first {
                        first = false;
                    } else {
                        ui.separator();
                    }

                    let ddi = dont_draw_idx.take_if(|i| i.0 == depth).map(|i| i.1).unwrap_or_default();
                    if let Some(opening) = draw_row(ui, depth, rewards, ddi) {
                        self.opening = Some(opening);
                    }
                }
            });

        if let Some(opening) = &mut self.opening {
            let chests = dungeon.rewards.get(&opening.chest_idx.0)
                .into_iter()
                .flatten()
                .enumerate()
                .filter(|(i,_)| opening.chest_idx.1.contains(i))
                .flat_map(|(_, c)| &c.items)
                .collect();

            if opening.show(ctx, chests) {
                dungeon.rewards.get_mut(&opening.chest_idx.0)
                    .into_iter()
                    .flat_map(|r| extract_if(r, &opening.chest_idx.1))
                    .flat_map(|c| c.items.into_iter())
                    .for_each(|item| stash.add(item));
                self.opening = None;
            }
        }

        if dungeon.rewards.is_empty() {
            self.open = false;
        }
    }
}

// TODO #[unstable(feature = "extract_if", reason = "recently added", issue = "43244")]
fn extract_if(vec: &mut Vec<RewardChest>, idx: &Vec<usize>) -> Vec<RewardChest> {
    let mut extracted = Vec::with_capacity(idx.len());
    let mut i_vec = 0;
    let mut i_idx = 0;
    while i_vec < vec.len() {
        if idx.contains(&i_idx) {
            extracted.push(vec.remove(i_vec));
        } else {
            i_vec += 1;
        }
        i_idx += 1;
    }
    extracted
}

fn draw_row(ui: &mut Ui, depth: u16, rewards: &Vec<RewardChest>, dont_draw_idx: Vec<usize>) -> Option<ChestOpening> {
    let mut opening = None;

    let mut open_5 = false;
    let mut open_5_rects = Vec::new();
    ui.horizontal(|ui| {
        ui.label(format!("depth {}", depth));
        if rewards.len() >= 5 {
            if ui.button("Open 5").clicked() {
                open_5 = true;
            }
        }
    });

    ui.horizontal_wrapped(|ui| {
        for (i, reward) in rewards.iter().enumerate() {
            if dont_draw_idx.contains(&i) {
                add_chest_button_placeholder(ui);
            } else {                
                let response = add_chest_button(ui);
                if response.clicked() {
                    opening = Some(ChestOpening {
                        start: SystemTime::now(),
                        chest_start_rects: vec![response.rect],
                        chest_idx: (depth, vec![i]),
                        seed: rand::rng().random(),
                        skipped: false,
                    });
                }
                text_in_rect(
                    ui,
                    RichText::new(reward.items.len().to_string()).color(Color32::WHITE).background_color(Color32::BLACK),
                    response.rect,
                    Align2::RIGHT_BOTTOM,
                );

                if open_5 && i < 5 {
                    open_5_rects.push(response.rect);
                }

            }
        }
    });

    if open_5 {        
        opening = Some(ChestOpening {
            start: SystemTime::now(),
            chest_start_rects: open_5_rects,
            chest_idx: (depth, vec![0,1,2,3,4]),
            seed: rand::rng().random(),
            skipped: false,
        });
    }

    opening
}

#[derive(Debug)]
struct ChestOpening {
    start: SystemTime,
    chest_start_rects: Vec<Rect>,
    chest_idx: (u16, Vec<usize>),
    seed: [u8; 32],
    skipped: bool,
}

impl ChestOpening {
    fn target(&self, ui: &Ui) -> Rect {
        Rect::from_center_size(ui.max_rect().center(), SIZE)
    }

    fn show(&mut self, ctx: &Context, items: Vec<&Item>) -> bool {
        ctx.request_repaint();

        let mut ui = Ui::new(
            ctx.clone(),
            "ChestOpeningUi".into(),
            UiBuilder::new().layer_id(LayerId::new(Order::Foreground, "ChestOpeningLayer".into())),
        );

        if self.skipped {
            return self.show_items(ctx, &items)
        }

        let elapsed = self.start.elapsed().unwrap().as_secs_f32();

        match elapsed {
            0.0..0.5 => self.move_to_center(&mut ui, inverse_lerp(0.0..=0.5, elapsed).unwrap()),
            0.5..1.2 => self.charge(&mut ui, inverse_lerp(0.5..=1.2, elapsed).unwrap()),
            1.2..1.5 => self.idle(&mut ui),
            1.5..2.5 => self.explode(&mut ui, &items, inverse_lerp(1.5..=2.5, elapsed).unwrap()),
            2.5..5.0 => self.fade(&mut ui, &items, inverse_lerp(2.5..=5.0, elapsed).unwrap()),
            5.0..7.5 => self.spin(&mut ui, &items, inverse_lerp(5.0..=7.5, elapsed).unwrap()),
            7.5..8.0 => self.collect(&mut ui, &items, inverse_lerp(7.5..=8.0, elapsed).unwrap()),
            _ => return self.show_items(ctx, &items),
        };


        if ui.put(
            Rect::from_center_size(ui.max_rect().center() + vec2(300., 300.), vec2(100., 40.)),
            Button::new("Skip")
        ).clicked() {
            self.skipped = true;
        }

        return false;
    }

    fn move_to_center(&self, ui: &mut Ui, t: f32) {
        for rect in &self.chest_start_rects {
            let pos = rect.lerp_towards(&self.target(ui), easing::quadratic_in_out(t));
            ui.put(pos, chest_img());
        }
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

    fn explode(&self, ui: &mut Ui, items: &Vec<&Item>, t: f32) {
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

    fn fade(&self, ui: &mut Ui, items: &Vec<&Item>, t: f32) {
        let target = self.target(ui);

        let full_size = ui.max_rect().size().min_elem() * 4.0;
        ui.painter().add(circle(target.center(), full_size, Color32::YELLOW.gamma_multiply(lerp(0.8..=0.0, easing::quadratic_in_out(t)))));

        let item_max_dist = 500.0.at_most(ui.max_rect().size().min_elem()) * 0.5;
        let item_rotation = lerp(PI..=1.5 * PI, t);
        spin_items(ui, items, item_max_dist, item_rotation);

        ui.put(target, open_chest_img());
    }

    fn spin(&self, ui: &mut Ui, items: &Vec<&Item>, t: f32) {
        let target = self.target(ui);

        let item_max_dist = 500.0.at_most(ui.max_rect().size().min_elem()) * 0.5;
        let item_rotation = lerp(1.5 * PI..=2.0 * PI, t);
        spin_items(ui, items, item_max_dist, item_rotation);

        ui.put(target, open_chest_img());
    }

    fn collect(&self, ui: &mut Ui, items: &Vec<&Item>, t: f32) {
        let target = self.target(ui);

        let item_max_dist = 500.0.at_most(ui.max_rect().size().min_elem()) * 0.5;
        spin_items(ui, items, lerp(item_max_dist..=0.0, easing::quadratic_in_out(t)), 0.);

        ui.put(target, open_chest_img());
    }

    fn show_items(&self, ctx: &Context, items: &Vec<&Item>) -> bool {
        let mut close = false;
        Window::new("Opened Chest")
            .title_bar(false)
            .auto_sized()
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .frame(Frame::window(&ctx.style()).fill(Color32::from_gray(48)))
            .show(ctx, |ui|
        {
            ScrollArea::vertical()
                .max_height(ctx.screen_rect().height() * 0.66)
                .max_width(0.)
                .show(ui, |ui|
            {
                for item in items {
                    ui.horizontal_top(|ui| {
                        ui.add_space(5.);
                        item.show(ui);
                        ui.add_space(2.);
                        ui.vertical(|ui| {
                            for modifier in &item.mods {
                                ui.horizontal(|ui| modifier.show_tooltip(ui));
                            }
                        });
                    });
                    ui.separator();
                }
                ui.vertical_centered(|ui| {
                    if ui.add(Button::new("Add to Loot").fill(Color32::from_gray(14))).clicked() {
                        close = true;
                    }
                });
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

fn spin_items(ui: &mut Ui, items: &Vec<&Item>, dist: f32, rot: f32) {
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
        let color = *COLORS.pick(&mut rng);

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

const SIZE: Vec2 = vec2(32., 32.);

fn chest_img<'a>() -> Image<'a> {
    Image::new(egui::include_image!("../../assets/icons/locked-chest.png"))
        .fit_to_exact_size(SIZE)
        .tint(Color32::GOLD)
}
fn open_chest_img<'a>() -> Image<'a> {
    Image::new(egui::include_image!("../../assets/icons/open-treasure-chest.png"))
        .fit_to_exact_size(SIZE)
        .tint(Color32::GOLD)
}
