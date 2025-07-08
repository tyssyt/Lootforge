use core::f64;
use std::time::Duration;

use crate::{dungeon::dungeon_data::DungeonTick, prelude::*};
use web_time::SystemTime;
use derive_more::Debug;

#[derive(Debug)]
pub struct FrameInfo {
    pub tick: bool,
    pub subframe: f32, // how far between frames, 0..1
    pub delay: f64, // in s
    pub catch_up: Option<u32>,

    #[debug(skip)]
    pub dungeon_tick: Option<DungeonTick>,
}

#[derive(Debug)]
pub struct Timekeeper {
    pub last_save_sim: SystemTime,
    last_save_wall: SystemTime,
    frames: u32,
    mode: Mode,
}
#[apply(Enum)]
enum Mode {
    Regular{
        last_frame_egui: f64,
        next_frame_egui: f64,
        delay: f64,
    },
    CatchingUp,
}

impl FrameInfo {
    pub fn anim(&self, ctx: &egui::Context, length: u32, remaining: u32, easing: fn(f32) -> f32) -> f32 {
        ctx.request_repaint();
        let cur = length as f32 - remaining as f32 + self.subframe;
        let progress = (cur / length as f32).clamp(0., 1.);
        easing(progress)
    }
}

impl Timekeeper {
    const FPS: u32 = 10;
    const FRAME_TIME: f64 = 1.0 / Self::FPS as f64;

    pub fn needs_save(&self) -> bool {
        match self.mode {
            Mode::Regular { last_frame_egui: _, next_frame_egui: _, delay: _ } => self.last_save_wall.elapsed().unwrap().as_secs() > 30,
            Mode::CatchingUp => false,
        }
    }

    pub fn save_now(&mut self) -> u64 {
        self.last_save_sim = self.last_save_sim + Duration::from_secs_f64(Self::FRAME_TIME * self.frames as f64);
        self.last_save_wall = SystemTime::now();
        self.frames = 0;
        self.last_save_sim.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64
    }

    // TODO clean this up a bit, gotten quite messy
    pub fn update(&mut self, ctx: &egui::Context) -> FrameInfo {
        let (time, predicted_dt) = ctx.input(|input| (input.time, input.predicted_dt));
        match self.mode {
            Mode::Regular { last_frame_egui, next_frame_egui, delay } => {
                if time >= next_frame_egui {
                    let last_frame_egui = time;
                    let delay = compute_delay(self.last_save_sim, self.frames);
                    if -delay > 5.*60. {
                        if -delay > 120.*60. {
                            info!("{:.2}h delay detected, beginning fast forward", -delay / 60. / 60.);
                        } else {
                            info!("{:.2}m delay detected, beginning fast forward", -delay / 60.);
                        }
                        ctx.request_repaint();
                        self.mode = Mode::CatchingUp;
                        let frames = (-delay / Timekeeper::FRAME_TIME) as u32;
                        return FrameInfo { tick: true, subframe: 0., delay, catch_up: Some(frames), dungeon_tick: None };
                    }
                    let next_frame_egui = compute_next_frame(time, delay);
                    request_repaint(ctx, next_frame_egui, time, predicted_dt);
                    let subframe = emath::inverse_lerp(last_frame_egui..=next_frame_egui, time).unwrap() as f32;
                    self.mode = Mode::Regular { last_frame_egui, next_frame_egui, delay };
                    FrameInfo { tick: true, subframe, delay, catch_up: None, dungeon_tick: None }
                } else {
                    request_repaint(ctx, next_frame_egui, time, predicted_dt);
                    let subframe = emath::inverse_lerp(last_frame_egui..=next_frame_egui, time).unwrap() as f32;
                    FrameInfo { tick: false, subframe, delay, catch_up: None, dungeon_tick: None }
                }
            },
            Mode::CatchingUp => {
                ctx.request_repaint();
                let delay = -compute_delay(self.last_save_sim, self.frames);
                if delay < 1. {                    
                    info!("fast forward completed");
                    self.last_save_wall = SystemTime::now() - Duration::from_secs(60); // force save
                    self.mode = Mode::Regular {
                        last_frame_egui: time,
                        next_frame_egui: compute_next_frame(time, delay),
                        delay
                    }
                }
                let frames = (delay / Timekeeper::FRAME_TIME) as u32;
                FrameInfo { tick: true, subframe: 0., delay, catch_up: Some(frames), dungeon_tick: None }
            },
        }
    }

    pub fn report_frames(&mut self, frames: u32) {
        self.frames += frames;
    }

}

fn compute_delay(last_save_wall: SystemTime, frames: u32) -> f64 {
    let simulated_time = frames as f64 * Timekeeper::FRAME_TIME;
    let actual_time = last_save_wall.elapsed()
        .map(|d| d.as_secs_f64())
        .unwrap_or(simulated_time);
    simulated_time - actual_time
}

fn compute_next_frame(time: f64, delay: f64) -> f64 {    
    // TODO compleletly rework
    let mut factor = delay * 0.05; // every second delay is a 5% speedup
    factor = factor.clamp(-0.1, 0.1);
    time + (Timekeeper::FRAME_TIME * (1. + factor))
}

fn request_repaint(ctx: &egui::Context, next_frame_egui: f64, time: f64, predicted_dt: f32) {
    let time_to_next_frame = (next_frame_egui - time) as f32;
    // egui thinks it's smart and subtracts predicted_dt from the time I give it...
    ctx.request_repaint_after_secs(time_to_next_frame + predicted_dt);
}

impl Default for Timekeeper {
    fn default() -> Self {
        let now = SystemTime::now();
        Self {
            last_save_sim: now,
            last_save_wall: now,
            frames: 0,
            mode: Mode::Regular {
                last_frame_egui: f64::NEG_INFINITY,
                next_frame_egui: f64::NEG_INFINITY,
                delay: 0.,
            }
        }
    }
}