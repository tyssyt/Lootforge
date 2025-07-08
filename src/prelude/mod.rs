#![allow(unused_imports)]

pub mod macros;
pub use macros::*;

pub use macro_rules_attribute::apply;

pub use strum::{VariantArray, IntoEnumIterator};
pub use smart_default::SmartDefault;
pub use rand::prelude::*;
pub use itertools::*;

pub use egui::*;
pub use egui_extras::*;
pub use emath::easing;

pub use std::ops;
pub use std::ops::{RangeInclusive, Deref};
pub use std::ptr;
pub use std::iter::once;
pub use std::rc::{Rc, Weak};
pub use std::cell::{Cell, RefCell};

pub use lootforge_macros::*;
pub use crate::util::*;

pub use log::{info, warn, error};
