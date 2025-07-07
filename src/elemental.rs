use std::iter;

use crate::prelude::*;
use Element::*;

#[repr(u8)]
#[apply(UnitEnum)]
#[derive(strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Element {
    Bleed = 1,
    Fracture = 2,
    Madness = 3,
    Void = 4,
}
impl Element {
    pub fn color(&self) -> Color32 {
        match self {
            Bleed => Color32::RED,
            Fracture => Color32::YELLOW,
            Madness => Color32::CYAN,
            Void => Color32::PURPLE,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Elemental<T> {
    pub bleed: T,
    pub fracture: T,
    pub madness: T,
    pub void: T,
}

impl<T: Copy> Elemental<T> {
    pub fn from(value: T) -> Self {
        Self {
            bleed: value,
            fracture: value,
            madness: value,
            void: value,
        }
    }
    pub fn with(&self, value: T, element: Element) -> Self {
        let mut clone = self.clone();
        clone.set(value, element);
        clone
    }
}
impl<T> Elemental<T> {
    pub fn get(&self, element: Element) -> &T {
        match element {
            Bleed  => &self.bleed,
            Fracture  => &self.fracture,
            Madness      => &self.madness,
            Void      => &self.void,
        }
    }
    pub fn get_mut(&mut self, element: Element) -> &mut T {
        match element {
            Bleed  => &mut self.bleed,
            Fracture  => &mut self.fracture,
            Madness      => &mut self.madness,
            Void      => &mut self.void,
        }
    }
    pub fn set(&mut self, value: T, element: Element) {
        match element {
            Bleed  => self.bleed  = value,
            Fracture  => self.fracture  = value,
            Madness      => self.madness      = value,
            Void      => self.void      = value,
        }
    }
    pub fn assign_cond(&mut self, from: Self, cond: Elemental<bool>) {
        if cond.bleed  { self.bleed  = from.bleed }
        if cond.fracture  { self.fracture  = from.fracture }
        if cond.madness      { self.madness      = from.madness }
        if cond.void      { self.void      = from.void }
    }
    pub fn choose<R: Rng + ?Sized>(&self, rng: &mut R) -> &T {
        self.get(*Element::VARIANTS.choose(rng).unwrap())
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.bleed, &self.fracture, &self.madness, &self.void].into_iter()
    }
}
impl<T: Copy + std::cmp::PartialOrd> Elemental<T> {
    pub fn at_least(self, min: T) -> Self {        
        Self {
            bleed: if self.bleed > min { self.bleed } else { min },
            fracture: if self.fracture > min { self.fracture } else { min },
            madness: if self.madness > min { self.madness } else { min },
            void:if self.void > min { self.void } else { min },
        }
    }
    pub fn max_elem(&self) -> &T {
        self.iter().reduce(|a, b| if a > b { a } else { b }).unwrap()
    }
    pub fn min_elem(&self) -> &T {
        self.iter().reduce(|a, b| if a < b { a } else { b }).unwrap()
    }
    pub fn max_idx(&self) -> Element {
        *self.iter().zip(Element::VARIANTS).reduce(|a, b| if a.0 > b.0 { a } else { b }).unwrap().1
    }
    pub fn min_idx(&self) -> Element {
        *self.iter().zip(Element::VARIANTS).reduce(|a, b| if a.0 < b.0 { a } else { b }).unwrap().1
    }
}
impl<T: NumExt> NumExt for Elemental<T> {
    fn at_least(self, lower_limit: Self) -> Self {
        Self {
            bleed: self.bleed.at_least(lower_limit.bleed),
            fracture: self.fracture.at_least(lower_limit.fracture),
            madness: self.madness.at_least(lower_limit.madness),
            void: self.void.at_least(lower_limit.void),
        }
    }
    fn at_most(self, upper_limit: Self) -> Self {
        Self {
            bleed: self.bleed.at_most(upper_limit.bleed),
            fracture: self.fracture.at_most(upper_limit.fracture),
            madness: self.madness.at_most(upper_limit.madness),
            void: self.void.at_most(upper_limit.void),
        }
    }
}
impl<T: Copy + ops::Add<Output = T>> Elemental<T> {
    pub fn sum(&self) -> T {
        self.bleed + self.fracture + self.madness + self.void
    }
}
impl<T: Copy + ops::Add<Output = T>> ops::Add for Elemental<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            bleed: self.bleed + rhs.bleed,
            fracture: self.fracture + rhs.fracture,
            madness: self.madness + rhs.madness,
            void: self.void + rhs.void,
        }
    }
}
impl<T: Copy + ops::Add<Output = T>> ops::Add<T> for Elemental<T> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Self {
            bleed: self.bleed + rhs,
            fracture: self.fracture + rhs,
            madness: self.madness + rhs,
            void: self.void + rhs,
        }
    }
}
impl<T: Copy + ops::Sub<Output = T>> ops::Sub for Elemental<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            bleed: self.bleed - rhs.bleed,
            fracture: self.fracture - rhs.fracture,
            madness: self.madness - rhs.madness,
            void: self.void - rhs.void,
        }
    }
}
impl<T: Copy + ops::Mul<Output = T>> ops::Mul for Elemental<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            bleed: self.bleed * rhs.bleed,
            fracture: self.fracture * rhs.fracture,
            madness: self.madness * rhs.madness,
            void: self.void * rhs.void,
        }
    }
}
impl<T: Copy + ops::Mul<Output = T>> ops::Mul<T> for Elemental<T> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self {
            bleed: self.bleed * rhs,
            fracture: self.fracture * rhs,
            madness: self.madness * rhs,
            void: self.void * rhs,
        }
    }
}
impl<T: Copy + ops::Div<Output = T>> ops::Div<T> for Elemental<T> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self {
            bleed: self.bleed / rhs,
            fracture: self.fracture / rhs,
            madness: self.madness / rhs,
            void: self.void / rhs,
        }
    }
}
impl<T: Copy + ops::Div<Output = T>> ops::Div for Elemental<T> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            bleed: self.bleed / rhs.bleed,
            fracture: self.fracture / rhs.fracture,
            madness: self.madness / rhs.madness,
            void: self.void / rhs.void,
        }
    }
}
impl<T: Copy + ops::Add<Output = T> + Default> iter::Sum for Elemental<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Default::default(), |a, b| a + b)
    }
}

impl<T: Copy> Copy for Elemental<T> {}
