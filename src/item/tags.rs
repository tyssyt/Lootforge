use enumset::EnumSetType;

use crate::{prelude::*, stash::stash::Stash};

#[derive(Debug, Default)]
pub struct ItemTags {
    rating: Cell<Rating>,

    pub equipped: Cell<bool>,
    any_wardrobe: Cell<bool>,
    wardrobe: [Cell<bool>; 9], // more precise info, like which char and then the exact slot?
                               // in workbench (store which position?)
}
impl ItemTags {
    pub fn from_rating(rating: Rating) -> Self {
        Self { rating: Cell::new(rating), ..Default::default() }
    }

    pub fn any_wardrobe(&self) -> bool {
        self.any_wardrobe.get()
    }
    pub fn wardrobes(&self) -> Vec<usize> {
        self.wardrobe.iter().enumerate()
            .filter(|(_, w)| w.get())
            .map(|(i, _)| i)
            .collect()
    }
    pub fn add_wardrobe(&self, i: usize) {
        self.wardrobe[i].set(true);
        self.any_wardrobe.set(true);
    }
    pub fn remove_wardrobe(&self, i: usize) {
        self.wardrobe[i].set(false);
        if self.wardrobe.iter().all(|b| !b.get()) {
            self.any_wardrobe.set(false);
        }
    }

    pub fn rating(&self) -> Rating {
        self.rating.get()
    }
    pub fn set_rating(&self, rating: Rating, stash: &mut Stash) {
        self.rating.set(rating);
        stash.invalidate_cached_filter();
    }
}

#[repr(u8)]
#[apply(UnitEnum)]
#[derive(Default, EnumSetType)]
#[enumset(no_super_impls)]
pub enum Rating {
    Favorite,
    Like,
    #[default]
    Neutral,
    Dislike,
    Trash,
}
impl Rating {    
    pub const SIZE: Vec2 = vec2(16., 16.);
    pub fn image(&self) -> Option<Image<'_>> {
        use Rating::*;
        let source = match *self {
            Favorite => include_image!("../../assets/icons/round-star.png"),
            Like     => include_image!("../../assets/icons/thumb-up.png"),
            Neutral  => return None,
            Dislike  => include_image!("../../assets/icons/thumb-down.png"),
            Trash   => include_image!("../../assets/icons/trash-can.png"),
        };
        Some(Image::new(source).fit_to_exact_size(Self::SIZE))
    }
}
